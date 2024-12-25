pub mod error;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Payload, Socket};
use tracing::{error, info};
use warhorse_protocol::*;
use crate::error::ClientError;

#[derive(Clone)]
pub enum WarhorseEvent {
    Hello,
    LoggedIn,
    Error(String),
    FriendRequests(Vec<Friend>),
    FriendsList(Vec<Friend>),
    BlockedList(Vec<Friend>),
    FriendRequestAccepted(Friend),
    ChatMessage(ChatMessage),
}

pub struct WarhorseClient {
    language: Language,
    socket_io: Client,
    pub pending_events: Arc<RwLock<VecDeque<WarhorseEvent>>>,
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
            .on(EVENT_RECEIVE_BLOCKED_USERS,
                {
                    let pending_events_clone = pending_events.clone();
                    move |payload, _socket| {
                        match payload {
                            Payload::Text(text) => {
                                if let Some(first) = text.first() {
                                    match json_to_vec::<Friend>(first.clone()) {
                                        Ok(blocked_list) => {
                                            if let Ok(mut event_queue) = pending_events_clone.write() {
                                                event_queue.push_back(WarhorseEvent::BlockedList(blocked_list));
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to parse blocked list: {:?}", e);
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
                })
            }
            Err(e) => {
                Err(ClientError(e.to_string()))
            }
        }
    }

    pub fn send_user_login_request(&self, user_login: UserLogin) -> Result<(), ClientError> {
        if let Err(e) = self.socket_io
            .emit(EVENT_SEND_USER_LOGIN, user_login.to_json()?) {
            return Err(ClientError(e.to_string()));
        }
        Ok(())
    }

    pub fn send_user_registration_request(&self, user_registration: UserRegistration) -> Result<(), ClientError> {
        if let Err(e) = self.socket_io
            .emit(EVENT_SEND_USER_REGISTER, user_registration.to_json()?) {
            return Err(ClientError(e.to_string()));
        }
        Ok(())
    }

    pub fn send_friend_request(&self, user_id: &str) -> Result<(), ClientError> {
        if let Err(e) = self.socket_io
            .emit(EVENT_SEND_FRIEND_REQUEST, FriendRequest {
                language: self.language.clone(),
                friend_id: user_id.to_string(),
            }.to_json()?) {
            return Err(ClientError(e.to_string()));
        }
        Ok(())
    }

    pub fn pump(&self) -> Vec<WarhorseEvent> {
        let mut events = Vec::new();
        if let Ok(mut event_queue) = self.pending_events.write() {
            while let Some(event) = event_queue.pop_front() {
                events.push(event);
            }
        }
        events
    }
}
