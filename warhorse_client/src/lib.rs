use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Payload};
use serde_json::json;
use tracing::error;
use horse_protocol::FriendsList;

pub struct HorseClient {
    socket_io: Client,
}

impl HorseClient {
    pub fn new(
        connection_string: &str,
        mut on_receive_friends_list: impl FnMut(&FriendsList) + Send + 'static,
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
                            let friends_list = FriendsList::from_json(first.clone())
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

        HorseClient {
            socket_io,
        }
    }

    pub fn send_friend_request(&self, friend_id: &str) {
        self.socket_io.emit("send-friend-request", json!({ "friend_id": friend_id }))
            .expect("Failed to send friend request");
    }
}
