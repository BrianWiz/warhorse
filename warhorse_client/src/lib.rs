pub mod error;

use rust_socketio::{ClientBuilder, Payload};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use tracing::error;

use crate::error::ClientError;
use warhorse_protocol::*;

// re-exports
pub use warhorse_protocol;

#[derive(Clone)]
pub enum WarhorseEvent {
    Hello,
    LoggedIn,
    Error(String),
    FriendsList(Vec<Friend>),
    FriendRequestReceived(Friend),
    FriendRequestAccepted(Friend),
    ChatMessage(ChatMessage),
}

pub struct WarhorseClient {
    // events we've received but haven't processed yet
    pending_receives: Arc<RwLock<VecDeque<WarhorseEvent>>>,
    // messages we've queued to send but haven't yet
    pending_sends: std::sync::mpsc::Sender<(String, serde_json::Value)>,
}

impl WarhorseClient {
    pub fn new(connection_string: &str) -> Result<Self, ClientError> {
        let pending_events = Arc::new(RwLock::new(VecDeque::new()));
        let socket_io = ClientBuilder::new(connection_string)
            .namespace("/")
            .on(EVENT_RECEIVE_USER_LOGIN, {
                let pending_events_clone = pending_events.clone();
                move |_payload, _socket| {
                    if let Ok(mut event_queue) = pending_events_clone.write() {
                        event_queue.push_back(WarhorseEvent::LoggedIn);
                    }
                }
            })
            .on(EVENT_RECEIVE_HELLO, {
                let pending_events_clone = pending_events.clone();
                move |payload, _socket| match payload {
                    Payload::Text(_) => {
                        if let Ok(mut event_queue) = pending_events_clone.write() {
                            event_queue.push_back(WarhorseEvent::Hello);
                        }
                    }
                    _ => {
                        error!("Unexpected payload: {:?}", payload);
                    }
                }
            })
            .on(EVENT_RECEIVE_ERROR, {
                let pending_events_clone = pending_events.clone();
                move |payload, _socket| match payload {
                    Payload::Text(text) => {
                        for line in text {
                            match RequestError::from_json(line.clone()) {
                                Ok(e) => {
                                    if let Ok(mut event_queue) = pending_events_clone.write() {
                                        event_queue.push_back(WarhorseEvent::Error(e.0));
                                    }
                                }
                                Err(e) => error!("Failed to parse error: {:?}", e),
                            }
                        }
                    }
                    _ => {
                        error!("Unexpected payload: {:?}", payload);
                    }
                }
            })
            .on(EVENT_RECEIVE_FRIENDS, {
                let pending_events_clone = pending_events.clone();
                move |payload, _socket| match payload {
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
            })
            .on(EVENT_RECEIVE_FRIEND_REQUESTS, {
                let pending_events_clone = pending_events.clone();
                move |payload, _socket| match payload {
                    Payload::Text(text) => {
                        if let Some(first) = text.first() {
                            match json_to_vec::<Friend>(first.clone()) {
                                Ok(mut friend_requests) => {
                                    if let Some(friend_request) = friend_requests.pop() {
                                        if let Ok(mut event_queue) = pending_events_clone.write() {
                                            event_queue.push_back(
                                                WarhorseEvent::FriendRequestReceived(
                                                    friend_request,
                                                ),
                                            );
                                        }
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
            })
            .on(EVENT_RECEIVE_FRIEND_REQUEST_ACCEPTED, {
                let pending_events_clone = pending_events.clone();
                move |payload, _socket| match payload {
                    Payload::Text(text) => {
                        if let Some(first) = text.first() {
                            match json_to_vec::<Friend>(first.clone()) {
                                Ok(mut friends) => {
                                    if let Some(friend) = friends.pop() {
                                        if let Ok(mut event_queue) = pending_events_clone.write() {
                                            event_queue.push_back(
                                                WarhorseEvent::FriendRequestAccepted(friend),
                                            );
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
            })
            .on(EVENT_RECEIVE_CHAT_MESSAGE, {
                let pending_events_clone = pending_events.clone();
                move |payload, _socket| match payload {
                    Payload::Text(text) => {
                        if let Some(first) = text.first() {
                            match ChatMessage::from_json(first.clone()) {
                                Ok(chat_message) => {
                                    if let Ok(mut event_queue) = pending_events_clone.write() {
                                        event_queue
                                            .push_back(WarhorseEvent::ChatMessage(chat_message));
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
            })
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
            pending_receives: pending_events,
            pending_sends: sender,
        })
    }

    pub fn send_user_login_request(
        &self,
        username: String,
        password: String,
    ) -> Result<(), ClientError> {
        let user_login = UserLogin {
            language: Language::English,
            identity: if Self::is_email_as_username(&username) {
                LoginUserIdentity::Email(username)
            } else {
                LoginUserIdentity::AccountName(username)
            },
            password,
        };

        let json = user_login.to_json()?;
        self.pending_sends
            .send((EVENT_SEND_USER_LOGIN.to_string(), json))
            .map_err(|e| ClientError(format!("Failed to queue login request: {:?}", e)))
    }

    pub fn send_user_registration_request(
        &self,
        account_name: String,
        password: String,
        display_name: String,
        email: String,
    ) -> Result<(), ClientError> {
        let user_registration = UserRegistration {
            account_name,
            password,
            email,
            display_name,
            language: Language::English,
        };

        let json = user_registration.to_json()?;
        self.pending_sends
            .send((EVENT_SEND_USER_REGISTER.to_string(), json))
            .map_err(|e| ClientError(format!("Failed to queue registration request: {:?}", e)))
    }

    pub fn send_friend_request(&self, friend_id: String) -> Result<(), ClientError> {
        let request = FriendRequest {
            language: Language::English,
            friend_id,
        };

        let json = request.to_json()?;
        self.pending_sends
            .send((EVENT_SEND_FRIEND_REQUEST.to_string(), json))
            .map_err(|e| ClientError(format!("Failed to queue friend request: {:?}", e)))
    }

    fn send_chat_message(&self, message: String, channel: ChatChannel) -> Result<(), ClientError> {
        let chat_message = SendChatMessage {
            language: Language::English,
            message,
            channel,
        };

        let json = chat_message.to_json()?;
        self.pending_sends
            .send((EVENT_SEND_CHAT_MESSAGE.to_string(), json))
            .map_err(|e| ClientError(format!("Failed to queue chat message: {:?}", e)))
    }

    pub fn send_whisper_message(
        &self,
        friend_id: String,
        message: String,
    ) -> Result<(), ClientError> {
        self.send_chat_message(message, ChatChannel::PrivateMessage(friend_id))
    }

    pub fn send_room_message(&self, room: String, message: String) -> Result<(), ClientError> {
        self.send_chat_message(message, ChatChannel::Room(room))
    }

    pub fn send_block_friend(&self, friend_id: String) -> Result<(), ClientError> {
        let request = BlockUserRequest {
            language: Language::English,
            user_id: friend_id,
        };

        let json = request.to_json()?;
        self.pending_sends
            .send((EVENT_SEND_USER_BLOCK.to_string(), json))
            .map_err(|e| ClientError(format!("Failed to queue block friend request: {:?}", e)))
    }

    pub fn send_unblock_friend(&self, friend_id: String) -> Result<(), ClientError> {
        let request = UnblockUserRequest {
            language: Language::English,
            user_id: friend_id,
        };

        let json = request.to_json()?;
        self.pending_sends
            .send((EVENT_SEND_USER_UNBLOCK.to_string(), json))
            .map_err(|e| ClientError(format!("Failed to queue unblock friend request: {:?}", e)))
    }

    pub fn send_accept_friend_request(&self, friend_id: String) -> Result<(), ClientError> {
        let request = AcceptFriendRequest {
            language: Language::English,
            friend_id,
        };

        let json = request.to_json()?;
        self.pending_sends
            .send((EVENT_SEND_FRIEND_REQUEST_ACCEPT.to_string(), json))
            .map_err(|e| ClientError(format!("Failed to queue accept friend request: {:?}", e)))
    }

    pub fn send_reject_friend_request(&self, friend_id: String) -> Result<(), ClientError> {
        let request = RejectFriendRequest {
            language: Language::English,
            friend_id,
        };

        let json = request.to_json()?;
        self.pending_sends
            .send((EVENT_SEND_FRIEND_REQUEST_REJECT.to_string(), json))
            .map_err(|e| ClientError(format!("Failed to queue reject friend request: {:?}", e)))
    }

    pub fn send_remove_friend(&self, friend_id: String) -> Result<(), ClientError> {
        let request = RemoveFriendRequest {
            language: Language::English,
            friend_id,
        };

        let json = request.to_json()?;
        self.pending_sends
            .send((EVENT_SEND_FRIEND_REMOVE.to_string(), json))
            .map_err(|e| ClientError(format!("Failed to queue remove friend request: {:?}", e)))
    }

    pub fn pump(&self) -> Vec<WarhorseEvent> {
        let mut events = Vec::new();
        if let Ok(mut event_queue) = self.pending_receives.write() {
            while let Some(event) = event_queue.pop_front() {
                events.push(event);
            }
        }
        events
    }

    fn is_email_as_username(input: &str) -> bool {
        input.contains('@')
    }
}
