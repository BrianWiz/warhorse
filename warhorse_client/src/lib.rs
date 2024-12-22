pub mod error;

use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Payload, Socket};
use tracing::{error, info};
use warhorse_protocol::*;
use crate::error::ClientError;

pub struct WarhorseClientCommands {
    pub user_login: Option<UserLogin>,
    pub user_registration: Option<UserRegistration>,
    pub friend_request: Option<FriendRequest>,
}

pub struct WarhorseClient {
    language: Language,
    socket_io: Client,
}

impl WarhorseClient {
    pub fn new(
        language: Language,
        connection_string: &str,
        mut on_receive_hello: impl FnMut(&mut WarhorseClientCommands, Language) + Send + 'static,
        mut on_receive_login_success: impl FnMut(&mut WarhorseClientCommands, Language) + Send + 'static,
        mut on_receive_error: impl FnMut(RequestError) + Send + 'static,
        mut on_receive_friends_list: impl FnMut(&Vec<Friend>) + Send + 'static,
        mut on_receive_blocked_list: impl FnMut(&Vec<UserPartial>) + Send + 'static,
        mut on_receive_friend_requests: impl FnMut(&Vec<Friend>) + Send + 'static,
        mut on_receive_friend_request_accepted: impl FnMut(&Friend) + Send + 'static,
    ) -> Self {
        let language = language.clone();
        let socket_io = ClientBuilder::new(connection_string)
            .namespace("/")
            .on(EVENT_RECEIVE_USER_LOGIN, move |_payload, socket| {
                let mut commands = WarhorseClientCommands {
                    user_login: None,
                    user_registration: None,
                    friend_request: None,
                };
                on_receive_login_success(&mut commands, language.clone());
                process_commands(commands, socket);
            })
            .on(EVENT_RECEIVE_HELLO, move |payload, socket| {
                match payload {
                    Payload::Text(_) => {
                        let mut commands = WarhorseClientCommands {
                            user_login: None,
                            user_registration: None,
                            friend_request: None,
                        };
                        on_receive_hello(&mut commands, language.clone());
                        process_commands(commands, socket);
                    }
                    _ => {
                        error!("Unexpected payload: {:?}", payload);
                    }
                }
            })
            .on(EVENT_RECEIVE_ERROR, move |payload, _socket| {
                match payload {
                    Payload::Text(text) => {
                        for line in text {
                            match RequestError::from_json(line.clone()) {
                                Ok(e) => on_receive_error(e),
                                Err(e) => error!("Failed to parse error: {:?}", e),
                            }
                        }
                    }
                    _ => {
                        error!("Unexpected payload: {:?}", payload);
                    }
                }
            })
            .on(EVENT_RECEIVE_FRIENDS, move |payload, _socket| {
                match payload {
                    Payload::Text(text) => {
                        if let Some(first) = text.first() {
                            match json_to_vec::<Friend>(first.clone()) {
                                Ok(friends) => {
                                    on_receive_friends_list(&friends);
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
            .on(EVENT_RECEIVE_BLOCKED_USERS, move |payload, _socket| {
                match payload {
                    Payload::Text(text) => {
                        if let Some(first) = text.first() {
                            match json_to_vec::<UserPartial>(first.clone()) {
                                Ok(blocked_list) => {
                                    on_receive_blocked_list(&blocked_list);
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
            })
            .on(EVENT_RECEIVE_FRIEND_REQUESTS, move |payload, _socket| {
                match payload {
                    Payload::Text(text) => {
                        if let Some(first) = text.first() {
                             match json_to_vec::<Friend>(first.clone()) {
                                 Ok(friends) => {
                                     on_receive_friend_requests(&friends);
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
            .on(EVENT_RECEIVE_FRIEND_REQUEST_ACCEPTED, move |payload, _socket| {
                match payload {
                    Payload::Text(text) => {
                        if let Some(first) = text.first() {
                            match json_to_vec::<Friend>(first.clone()) {
                                Ok(mut friends) => {
                                    if let Some(friend) = friends.pop() {
                                        on_receive_friend_request_accepted(&friend);
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
            .on(EVENT_RECEIVE_CHAT_MESSAGE, move |payload, _socket| {

            })
            .connect()
            .expect("Connection failed");

        WarhorseClient {
            socket_io,
            language,
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
}

fn process_commands(commands: WarhorseClientCommands, socket: Socket) {
    if let Some(user_login) = commands.user_login {
        info!("Sending user login request");
        match user_login.to_json() {
            Ok(json) => {
                if let Err(e) = socket.emit(EVENT_SEND_USER_LOGIN, json) {
                    error!("Failed to send user login request: {:?}", e);
                }
            }
            Err(e) => {
                error!("Failed to serialize user login request: {:?}", e);
            }
        }
    }

    if let Some(user_registration) = commands.user_registration {
        info!("Sending user registration request");
        match user_registration.to_json() {
            Ok(json) => {
                if let Err(e) = socket.emit(EVENT_SEND_USER_REGISTER, json) {
                    error!("Failed to send user registration request: {:?}", e);
                }
            }
            Err(e) => {
                error!("Failed to serialize user registration request: {:?}", e);
            }
        }
    }

    if let Some(friend_request) = commands.friend_request {
        info!("Sending friend request to: {}", friend_request.friend_id);
        match friend_request.to_json() {
            Ok(json) => {
                if let Err(e) = socket.emit(EVENT_SEND_FRIEND_REQUEST, json) {
                    error!("Failed to send friend request: {:?}", e);
                }
            }
            Err(e) => {
                error!("Failed to serialize friend request: {:?}", e);
            }
        }
    }
}
