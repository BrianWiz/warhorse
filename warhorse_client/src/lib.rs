pub mod error;

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Payload};
use serde_json::json;
use tracing::{error, info};

use warhorse_protocol::*;
use crate::error::ClientError;

// re-exports
pub use warhorse_protocol;

#[derive(Clone)]
pub enum WarhorseEvent {
    Hello,
    LoggedIn,
    Error(String),
    FriendRequests(Vec<Friend>),
    FriendsList(Vec<Friend>),
    FriendRequestAccepted(Friend),
    ChatMessage(ChatMessage),
}

pub struct WarhorseClient {
    language: Language,
    socket_io: Client,
    // events we've received but haven't processed yet
    pub pending_events: Arc<RwLock<VecDeque<WarhorseEvent>>>,
    // messages we've queued to send but haven't yet
    pending_messages: Arc<RwLock<VecDeque<(String, serde_json::Value)>>>
}

impl WarhorseClient {
    pub fn new(
        language: Language,
        connection_string: &str,
    ) -> Result<Self, ClientError> {
        let language = language.clone();
        let pending_events = Arc::new(RwLock::new(VecDeque::new()));

        let socket_io = ClientBuilder::new(connection_string)
            .namespace("/")
            .on(EVENT_RECEIVE_USER_LOGIN,
                {
                    let pending_events_clone = pending_events.clone();
                    move |_payload, _socket| {
                        if let Ok(mut event_queue) = pending_events_clone.write() {
                            event_queue.push_back(WarhorseEvent::LoggedIn);
                        }
                    }
                }
            )
            .on(EVENT_RECEIVE_HELLO,
            {
                let pending_events_clone = pending_events.clone();
                move |payload, _socket| {
                    match payload {
                        Payload::Text(_) => {
                            if let Ok(mut event_queue) = pending_events_clone.write() {
                                event_queue.push_back(WarhorseEvent::Hello);
                            }
                        }
                        _ => {
                            error!("Unexpected payload: {:?}", payload);
                        }
                    }
                }
            })
            .on(EVENT_RECEIVE_ERROR,
                {
                    let pending_events_clone = pending_events.clone();
                    move |payload, _socket| {
                        match payload {
                            Payload::Text(text) => {
                                for line in text {
                                    match RequestError::from_json(line.clone()) {
                                        Ok(e) => {
                                            if let Ok(mut event_queue) = pending_events_clone.write() {
                                                event_queue.push_back(WarhorseEvent::Error(e.0));
                                            }
                                        },
                                        Err(e) => error!("Failed to parse error: {:?}", e),
                                    }
                                }
                            }
                            _ => {
                                error!("Unexpected payload: {:?}", payload);
                            }
                        }
                    }
                })
                .on(EVENT_RECEIVE_FRIENDS,
                {
                    let pending_events_clone = pending_events.clone();
                    move |payload, _socket| {
                        match payload {
                            Payload::Text(text) => {
                                if let Some(first) = text.first() {
                                    match json_to_vec::<Friend>(first.clone()) {
                                        Ok(friends) => {
                                            if let Ok(mut event_queue) = pending_events_clone.write() {
                                                event_queue.push_back(WarhorseEvent::FriendsList(friends));
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to parse friends list: {:?}", e);
                                        }
                                    }
                                }
                            }
                            _ => {
                                error!("Unexpected payload: {:?}", payload);
                            }
                        }
                    }
                }
            )
            .on(EVENT_RECEIVE_FRIEND_REQUESTS,
                {
                    let pending_events_clone = pending_events.clone();
                    move |payload, _socket| {
                        match payload {
                            Payload::Text(text) => {
                                if let Some(first) = text.first() {
                                    match json_to_vec::<Friend>(first.clone()) {
                                        Ok(friends) => {
                                            if let Ok(mut event_queue) = pending_events_clone.write() {
                                                event_queue.push_back(WarhorseEvent::FriendRequests(friends));
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to parse friend requests: {:?}", e);
                                        }
                                    }
                                }
                            }
                            _ => {
                                error!("Unexpected payload: {:?}", payload);
                            }
                        }
                    }
                }
            )
            .on(EVENT_RECEIVE_FRIEND_REQUEST_ACCEPTED,
                {
                    let pending_events_clone = pending_events.clone();
                    move |payload, _socket| {
                        match payload {
                            Payload::Text(text) => {
                                if let Some(first) = text.first() {
                                    match json_to_vec::<Friend>(first.clone()) {
                                        Ok(mut friends) => {
                                            if let Some(friend) = friends.pop() {
                                                if let Ok(mut event_queue) = pending_events_clone.write() {
                                                    event_queue.push_back(WarhorseEvent::FriendRequestAccepted(friend));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to parse friend request accepted: {:?}", e);
                                        }
                                    }
                                }
                            }
                            _ => {
                                error!("Unexpected payload: {:?}", payload);
                            }
                        }
                    }
                }
            )
            .on(EVENT_RECEIVE_CHAT_MESSAGE,
        {
                    let pending_events_clone = pending_events.clone();
                    move |payload, _socket| {
                        match payload {
                            Payload::Text(text) => {
                                if let Some(first) = text.first() {
                                    match json_to_vec::<ChatMessage>(first.clone()) {
                                        Ok(messages) => {
                                            for message in messages {
                                                if let Ok(mut event_queue) = pending_events_clone.write() {
                                                    event_queue.push_back(WarhorseEvent::ChatMessage(message));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to parse chat message: {:?}", e);
                                        }
                                    }
                                }
                            }
                            _ => {
                                error!("Unexpected payload: {:?}", payload);
                            }
                        }
                    }
                }
            )
            .connect();

        match socket_io {
            Ok(socket_io) => {
                Ok(WarhorseClient {
                    socket_io,
                    language,
                    pending_events,
                    pending_messages: Arc::new(RwLock::new(VecDeque::new())),
                })
            }
            Err(e) => {
                Err(ClientError(e.to_string()))
            }
        }
    }

    pub fn send_user_login_request(&self, user_login: UserLogin) -> Result<(), ClientError> {
        let json = user_login.to_json()?;
        if let Ok(mut messages) = self.pending_messages.write() {
            messages.push_back((EVENT_SEND_USER_LOGIN.to_string(), json));
            Ok(())
        } else {
            Err(ClientError("Failed to queue login request".to_string()))
        }
    }

    pub fn send_user_registration_request(&self, user_registration: UserRegistration) -> Result<(), ClientError> {
        let json = user_registration.to_json()?;
        if let Ok(mut messages) = self.pending_messages.write() {
            messages.push_back((EVENT_SEND_USER_REGISTER.to_string(), json));
            Ok(())
        } else {
            Err(ClientError("Failed to queue registration request".to_string()))
        }
    }

    pub fn send_friend_request(&self, user_id: &str) -> Result<(), ClientError> {
        let json = json!(user_id);
        if let Ok(mut messages) = self.pending_messages.write() {
            messages.push_back((EVENT_SEND_FRIEND_REQUEST.to_string(), json));
            Ok(())
        } else {
            Err(ClientError("Failed to queue friend request".to_string()))
        }
    }

    pub fn pump(&self) -> Vec<WarhorseEvent> {
        let mut events = Vec::new();

        // send any pending messages
        if let Ok(mut messages) = self.pending_messages.write() {
            while let Some((event, json)) = messages.pop_front() {
                match self.socket_io.emit(event, json) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to send message: {:?}", e);
                    }
                }
            }
        }

        if let Ok(mut event_queue) = self.pending_events.write() {
            while let Some(event) = event_queue.pop_front() {
                events.push(event);
            }
        }
        events
    }
}
