use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Payload};
use serde_json::json;
use tracing::error;
use warhorse_protocol::{json_to_vec, Friend};

pub struct WarhorseClient {
    socket_io: Client,
}

impl WarhorseClient {
    pub fn new(
        connection_string: &str,
        mut on_receive_friends_list: impl FnMut(&Vec<Friend>) + Send + 'static,
    ) -> Self {
        let socket_io = ClientBuilder::new(connection_string)
            .namespace("/")
            .on("auth", move |payload, _socket| {
                println!("Authenticated: {:?}", payload);
            })
            .on("friends-list", move |payload, _socket| {
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
            .on("error", |err, _| eprintln!("Error: {:#?}", err))
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
