mod data_service;
mod database;
mod utils;

use std::sync::Arc;
use data_service::DataService;
use database::{db_in_memory::InMemoryDatabase, Database};
use tokio::sync::{Mutex, RwLock};
use std::collections::HashMap;
use std::error::Error;
use axum::routing::get;
use serde_json::Value;
use socketioxide::{
    extract::{Data, SocketRef},
    SocketIo,
};
use socketioxide::operators::BroadcastOperators;
use socketioxide::socket::Sid;
use horse_protocol::*;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

type SocketId = Sid;

pub struct HorseServer<T>
    where T: Database + Send + Sync + 'static
{
    data_service: DataService<T>,
    user_sockets: HashMap<UserId, SocketId>,
    io: SocketIo,

    // temp until we have an actual database connected
    temp_next_user_id: usize,
}

impl<T> HorseServer<T>
    where T: Database + Send + Sync + 'static
{
    pub fn new(io: SocketIo) -> Self {
        Self {
            io,
            user_sockets: HashMap::new(),
            temp_next_user_id: 0,
            data_service: DataService::new(T::new()),
        }
    }

    /// Gets the online status of a user
    fn get_online_status(&self, user_id: UserId) -> FriendStatus {
        if self.user_sockets.contains_key(&user_id) {
            FriendStatus::Online
        } else {
            FriendStatus::Offline
        }
    }

    /// Gets a room by its ID
    pub fn get_room(&self, room_id: RoomId) -> BroadcastOperators {
        self.io.to(room_id)
    }

    /// Gets a socket by its ID
    pub fn get_socket(&self, socket_id: SocketId) -> Option<SocketRef> {
        self.io.get_socket(socket_id)
    }

    /// Gets the socket ID associated with a user
    pub fn get_socket_id(&self, user_id: UserId) -> Result<SocketId, Box<dyn Error>> {
        match self.user_sockets.get(&user_id) {
            Some(socket_id) => Ok(socket_id.clone()),
            None => Err(format!("{} is not connected", user_id))?,
        }
    }

    /// Registers a user's socket
    pub async fn register_user(&mut self, 
        user_id: UserId,
        account_name: String,
        email: String,
        display_name: String,
        password: String,
        socket_id: SocketId,
    ) -> Result<(), Box<dyn Error>> {
        self.data_service.create_user(
            user_id.clone(),
            account_name,
            email,
            display_name,
            password,
        )?;

        self.user_sockets.insert(user_id, socket_id);
        Ok(())
    }

    /// Removes a user's socket
    pub async fn remove_user(&mut self, user_id: &str) {
        self.user_sockets.remove(user_id);
    }

    /// Sends a private message to a specific user
    pub fn send_chat_message(&self, sender_id: UserId, message: SendChatMessage) -> Result<(), Box<dyn Error>> {
        let serialized_message = message.to_json()?;
        match message.channel {
            ChatChannel::PrivateMessage(user_id) => {
                if self.are_friends(sender_id.clone(), user_id.clone()) {
                    let socket_id = self.get_socket_id(user_id.clone())?;
                    if let Some(socket) = self.get_socket(socket_id) {
                        socket.emit("chat-message", &serialized_message)?;
                    } else {
                        Err(format!("{} is not connected", user_id))?;
                    }
                } else {
                    Err(format!("{} is not friends with {}", sender_id, user_id))?;
                }
            },
            ChatChannel::Room(room_id) => {
                if self.user_in_room(sender_id.clone(), room_id.clone()) {
                    self.get_room(room_id)
                        .emit("chat-message", &serialized_message)?;
                } else {
                    Err(format!("{} is not in room {}", sender_id, room_id))?;
                }
            }
        }

        Ok(())
    }

    /// Whether two users are friends
    fn are_friends(&self, user_id: UserId, friend_id: UserId) -> bool {
        true // Temp until we have an actual database connected
    }

    /// Whether a user is in a room
    fn user_in_room(&self, user_id: UserId, room_id: RoomId) -> bool {
        if self.room_exists(room_id) {
            true // Temp until we have an actual database connected
        } else {
            false
        }
    }

    /// Whether a room exists
    fn room_exists(&self, room_id: RoomId) -> bool {
        true // Temp until we have an actual database connected
    }

    /// Gets the user ID of the logged in user associated with a socket
    pub fn get_logged_in_user_id(&self, socket_id: SocketId) -> Option<UserId> {
        self.user_sockets.iter().find_map(|(user_id, id)| {
            if id == &socket_id {
                Some(user_id.clone())
            } else {
                None
            }
        })
    }

    /// Gets the friends list for a user
    pub fn get_friends_list(&self, user_id: String) -> FriendsList {
        // Temp until we have an actual database connected
        FriendsList::from(vec![
            Friend {
                id: "1".to_string(),
                display_name: "John".to_string(),
                status: self.get_online_status("1".to_string()),
            },
            Friend {
                id: "2".to_string(),
                display_name: "Jane".to_string(),
                status: self.get_online_status("2".to_string()),
            },
        ])
    }
}

pub fn listen_for_chat_messages<T: Database + Send + Sync + 'static>(socket_ref: &SocketRef, server: Arc<Mutex<HorseServer<T>>>) {
    socket_ref.on("chat-message", move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match SendChatMessage::from_json(data) {
                Ok(data) => {
                    match server.lock().await.get_logged_in_user_id(socket.id) {
                        Some(sender_id) => {
                            if let Err(e) = server.lock().await.send_chat_message(sender_id, data) {
                                error!(ns = socket.ns(), ?socket.id, ?e, "Failed to send chat message");
                            }
                        },
                        None => {
                            error!(ns = socket.ns(), ?socket.id, "Failed to get user ID");
                        }
                    }
                },
                Err(e) => {
                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to parse chat message");
                }
            };
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let (layer, io) = SocketIo::new_layer();
    let horse_server = Arc::new(Mutex::new(HorseServer::<InMemoryDatabase>::new(io)));

    let horse_server_clone = horse_server.clone();
    horse_server_clone.lock().await.io.ns("/", move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            handle_connection(socket, data, horse_server_clone.clone()).await;
        }
    });


    let app = axum::Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(layer);

    info!("Starting server");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn handle_connection<T: Database + Send + Sync + 'static>(socket: SocketRef, data: Value, server: Arc<Mutex<HorseServer<T>>>) {

    info!(ns = socket.ns(), ?socket.id, "Socket.IO connected");

    let user_id = if let Some(user_id) = data.get("user_id").and_then(|v| v.as_str()) {
        user_id.to_string()
    } else {
        error!(ns = socket.ns(), ?socket.id, "No user ID provided. Cannot auth.");
        return;
    };

    let password = if let Some(password) = data.get("password").and_then(|v| v.as_str()) {
        password.to_string()
    } else {
        error!(ns = socket.ns(), ?socket.id, "No password provided. Cannot auth.");
        return;
    };

    let account_name = if let Some(account_name) = data.get("account_name").and_then(|v| v.as_str()) {
        account_name.to_string()
    } else {
        error!(ns = socket.ns(), ?socket.id, "No account name provided. Cannot auth.");
        return;
    };

    let email = if let Some(email) = data.get("email").and_then(|v| v.as_str()) {
        email.to_string()
    } else {
        error!(ns = socket.ns(), ?socket.id, "No email provided. Cannot auth.");
        return;
    };

    let display_name = if let Some(display_name) = data.get("display_name").and_then(|v| v.as_str()) {
        display_name.to_string()
    } else {
        error!(ns = socket.ns(), ?socket.id, "No display name provided. Cannot auth.");
        return;
    };

    let user_registration_error = server.lock().await.data_service.create_user(
        user_id.clone(),
        account_name.clone(),
        email.clone(),
        display_name.clone(),
        password.clone(),
    );

    match user_registration_error {
        Ok(()) => {
            info!(ns = socket.ns(), ?socket.id, "Registered user");
            socket.emit("auth", &data).ok();
        },
        Err(e) => {
            info!(ns = socket.ns(), ?socket.id, ?e, "Failed to register");
            if let Ok(json) = LoginResponse::Failure(e.to_string()).to_json() {
                socket.emit("auth-error", &json).ok();
            } else {
                error!(ns = socket.ns(), ?socket.id, "Failed to serialize auth error");
            }
            socket.disconnect();
            return;
        }
    }

    if let Ok(serialized_message) = server.lock().await.get_friends_list(user_id.clone()).to_json() {
        socket.emit("friends-list", &serialized_message.to_string()).ok();
    } else {
        error!(ns = socket.ns(), ?socket.id, "Failed to serialize friends list");
    }

    listen_for_chat_messages(&socket, server.clone());

    // Handle disconnection
    let server_clone = server.clone();
    socket.on_disconnect(move || {
        let server = server_clone.clone();
        let user_id = user_id.clone();
        async move {
            server.lock().await.remove_user(&user_id).await;
        }
    });
}
