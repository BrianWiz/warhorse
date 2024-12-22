use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Payload};
use serde_json::json;
use tracing::{error, info};
use warhorse_protocol::*;

pub struct WarhorseClient {
    socket_io: Client,
}

impl WarhorseClient {
    pub fn new(
        connection_string: &str,
        mut on_receive_friends_list: impl FnMut(&Vec<Friend>) + Send + 'static,
        mut on_receive_blocked_list: impl FnMut(&Vec<UserPartial>) + Send + 'static,
        mut on_receive_friend_requests: impl FnMut(&Vec<Friend>) + Send + 'static,
    ) -> Self {
        let socket_io = ClientBuilder::new(connection_string)
            .namespace("/")
            .on(EVENT_RECEIVE_USER_LOGIN, move |payload, _socket| {
                info!("Authenticated: {:?}", payload);
            })
            .on(EVENT_RECEIVE_USER_LOGIN_ERROR, move |payload, _socket| {
                error!("Login error: {:?}", payload);
            })
            .on(EVENT_RECEIVE_USER_REGISTER_ERROR, move |payload, _socket| {
                error!("Registration error: {:?}", payload);
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
            .on("error", |err, _| error!("Error: {:#?}", err))
            .connect()
            .expect("Connection failed");

        WarhorseClient {
            socket_io,
        }
    }

    pub fn send_friend_request(&self, friend_id: &str) {
        self.socket_io.emit("send-friend-request", json!({ "friend_id": friend_id }))
            .expect("Failed to send friend request");
    }
}
