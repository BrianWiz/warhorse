pub mod error;

use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Payload};
use tracing::{error, info};
use warhorse_protocol::*;
use crate::error::ClientError;

pub struct WarhorseClient {
    language: Language,
    socket_io: Client,
}

impl WarhorseClient {
    pub fn new(
        language: Language,
        connection_string: &str,
        mut on_receive_error: impl FnMut(RequestError) + Send + 'static,
        mut on_receive_friends_list: impl FnMut(&Vec<Friend>) + Send + 'static,
        mut on_receive_blocked_list: impl FnMut(&Vec<UserPartial>) + Send + 'static,
        mut on_receive_friend_requests: impl FnMut(&Vec<Friend>) + Send + 'static,
        mut on_receive_friend_request_accepted: impl FnMut(&Friend) + Send + 'static,
    ) -> Self {
        let socket_io = ClientBuilder::new(connection_string)
            .namespace("/")
            .on(EVENT_RECEIVE_USER_LOGIN, move |payload, _socket| {
                info!("Authenticated: {:?}", payload);
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
                            let friends_list = json_to_vec::<Friend>(first.clone())
                                .expect("Failed to parse friends list");
                            on_receive_friends_list(&friends_list);
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
                            let blocked_list = json_to_vec::<UserPartial>(first.clone())
                                .expect("Failed to parse blocked list");
                            on_receive_blocked_list(&blocked_list);
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
                            let friend_requests = json_to_vec::<Friend>(first.clone())
                                .expect("Failed to parse friend requests");
                            on_receive_friend_requests(&friend_requests);
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
                            let friend = json_to_vec::<Friend>(first.clone())
                                .expect("Failed to parse friend request accepted");
                            on_receive_friend_request_accepted(&friend[0]);
                        }
                    }
                    _ => {
                        error!("Unexpected payload: {:?}", payload);
                    }
                }
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
