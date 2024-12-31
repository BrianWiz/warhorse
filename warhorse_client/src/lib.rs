pub mod error;

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use rust_socketio::{ClientBuilder, Payload};
use serde_json::json;
use tracing::error;

use warhorse_protocol::*;
use crate::error::ClientError;

// re-exports
pub use warhorse_protocol;

#[derive(Clone)]
pub enum WarhorseEvent {
    Hello,
    LoggedIn,
    Error(String),
    FriendsList(Vec<Friend>),
    FriendRequestAccepted(Friend),
    ChatMessage(ChatMessage),
}

pub struct WarhorseClient {
    // events we've received but haven't processed yet
    pending_events: Arc<RwLock<VecDeque<WarhorseEvent>>>,
    // messages we've queued to send but haven't yet
    sender: std::sync::mpsc::Sender<(String, serde_json::Value)>,
}

impl WarhorseClient {
    pub fn new(
        connection_string: &str,
    ) -> Result<Self, ClientError> {
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
                                    match ChatMessage::from_json(first.clone()) {
                                        Ok(chat_message) => {
                                            if let Ok(mut event_queue) = pending_events_clone.write() {
                                                event_queue.push_back(WarhorseEvent::ChatMessage(chat_message));
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

        if let Err(e) = socket_io {
            return Err(ClientError(format!("Failed to connect: {:?}", e)));
        }

        let socket_io = Arc::new(socket_io.unwrap());
        let socket_io_clone = socket_io.clone();

        // Create a channel for sending socket messages
        let (sender, receiver) = std::sync::mpsc::channel::<(String, serde_json::Value)>();
        
        // Start a background thread for handling socket emissions
        std::thread::spawn(move || {
            while let Ok((event, json)) = receiver.recv() {
                match socket_io_clone.emit(event, json) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to send message: {:?}", e);
                    }
                }
            }
        });

        Ok(WarhorseClient {
            pending_events,
            sender,
        })
    }

    pub fn send_user_login_request(&self, user_login: UserLogin) -> Result<(), ClientError> {
        let json = user_login.to_json()?;
        if let Err(e) = self.sender.send((EVENT_SEND_USER_LOGIN.to_string(), json)) {
            return Err(ClientError(format!("Failed to queue login request: {:?}", e)));
        }
        Ok(())
    }

    pub fn send_user_registration_request(&self, user_registration: UserRegistration) -> Result<(), ClientError> {
        let json = user_registration.to_json()?;
        if let Err(e) = self.sender.send((EVENT_SEND_USER_REGISTER.to_string(), json)) {
            return Err(ClientError(format!("Failed to queue registration request: {:?}", e)));
        }
        Ok(())
    }

    pub fn send_friend_request(&self, user_id: &str) -> Result<(), ClientError> {
        let json = json!(user_id);
        if let Err(e) = self.sender.send((EVENT_SEND_FRIEND_REQUEST.to_string(), json)) {
            return Err(ClientError(format!("Failed to queue friend request: {:?}", e)));
        }
        Ok(())
    }

    pub fn send_chat_message(&self, chat_message: SendChatMessage) -> Result<(), ClientError> {
        let json = chat_message.to_json()?;
        if let Err(e) = self.sender.send((EVENT_SEND_CHAT_MESSAGE.to_string(), json)) {
            return Err(ClientError(format!("Failed to queue chat message: {:?}", e)));
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
