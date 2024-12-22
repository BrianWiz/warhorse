use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use serde_json::Value;
use socketioxide::{
    extract::{Data, SocketRef},
    SocketIo,
};
use socketioxide::operators::BroadcastOperators;
use socketioxide::socket::Sid;
use warhorse_protocol::*;
use tracing::{error, info};

use crate::data_service::DataService;
use crate::database::Database;
use crate::error::ServerError;

type SocketId = Sid;

pub struct WarhorseServer<T>
where T: Database + Send + Sync + 'static
{
    data_service: DataService<T>,
    user_sockets: HashMap<UserId, SocketId>,
    io: SocketIo,
}

impl<T> WarhorseServer<T>
where T: Database + Send + Sync + 'static
{
    pub fn new(io: SocketIo, database_connection_string: &str) -> Self {
        Self {
            io,
            user_sockets: HashMap::new(),
            data_service: DataService::new(T::new(database_connection_string)),
        }
    }

    /// Gets the Socket.IO instance
    pub fn io(&self) -> &SocketIo {
        &self.io
    }

    /// Gets the online status of a user
    fn get_online_status(&self, user_id: UserId) -> FriendOnlineStatus {
        if self.user_sockets.contains_key(&user_id) {
            FriendOnlineStatus::Online
        } else {
            FriendOnlineStatus::Offline
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
    pub fn get_socket_id(&self, user_id: UserId) -> Result<SocketId, ServerError> {
        match self.user_sockets.get(&user_id) {
            Some(socket_id) => Ok(socket_id.clone()),
            None => Err(format!("{} is not connected", user_id))?,
        }
    }

    /// Logs in a user
    pub async fn login_user(
        &mut self,
        req: UserLogin,
        socket_id: SocketId
    ) -> Result<(), ServerError> {
        let user = match req.identity {
            LoginUserIdentity::AccountName(account_name) => {
                self.data_service.users_get_by_account_name(&account_name)
            },
            LoginUserIdentity::Email(email) => {
                self.data_service.users_get_by_email(&email)
            },
        };

        if let Some(user) = user {
            self.user_sockets.insert(user.id, socket_id);
            Ok(())
        } else {
            Err(crate::i18n::invalid_login(req.language))?
        }
    }

    /// Registers a new user and logs them in
    pub async fn register_user(
        &mut self,
        req: UserRegistration,
        socket_id: SocketId
    ) -> Result<(), ServerError> {
        let user_id = self.data_service.create_user(req)?;
        self.user_sockets.insert(user_id, socket_id);
        Ok(())
    }

    /// Removes a user's socket
    pub async fn remove_user(&mut self, user_id: &str) {
        self.user_sockets.remove(user_id);
    }

    /// Sends a private message to a specific user
    pub fn send_chat_message(&self, sender_id: UserId, message: SendChatMessage) -> Result<(), ServerError> {
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

    /// Gets the friends list of a user and their online status
    pub fn get_friends_list(&self, user_id: UserId) -> Vec<Friend> {
        let mut friends_list = self.data_service.friends_get(user_id);
        for friend in friends_list.iter_mut() {
            friend.status = self.get_online_status(friend.id.clone());
        }
        friends_list
    }
}

pub fn listen_for_chat_messages<T: Database + Send + Sync + 'static>(socket_ref: &SocketRef, server: Arc<Mutex<WarhorseServer<T>>>) {
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
                            socket.disconnect().ok();
                        }
                    }
                },
                Err(e) => {
                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to parse chat message");
                    socket.disconnect().ok();
                }
            };
        }
    });
}

pub fn listen_for_user_login<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on("auth-login", move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match UserLogin::from_json(data) {
                Ok(data) => {
                    match server.lock().await.login_user(data, socket.id).await {
                        Ok(_) => {
                            info!(ns = socket.ns(), ?socket.id, "User logged in");
                        },
                        Err(e) => {
                            error!(ns = socket.ns(), ?socket.id, ?e, "Failed to log in user");
                            socket.disconnect().ok();
                        }
                    }
                },
                Err(e) => {
                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to parse login data");
                    socket.disconnect().ok();
                }
            }
        }
    });
}

pub fn listen_for_user_registration<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on("auth-register", move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match UserRegistration::from_json(data) {
                Ok(data) => {
                    match server.lock().await.register_user(data, socket.id).await {
                        Ok(_) => {
                            info!(ns = socket.ns(), ?socket.id, "User registered");
                        },
                        Err(e) => {
                            error!(ns = socket.ns(), ?socket.id, ?e, "Failed to register user");
                        }
                    }
                },
                Err(e) => {
                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to parse registration data");
                }
            }
        }
    });
}

pub fn handle_user_disconnect<T: Database + Send + Sync + 'static>(
    socket: SocketRef,
    user_id: UserId,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    let server_clone = server.clone();
    socket.on_disconnect(move || {
        let server = server_clone.clone();
        let user_id = user_id.clone();
        async move {
            server.lock().await.remove_user(&user_id).await;
        }
    });
}

pub async fn handle_connection<T: Database + Send + Sync + 'static>(
    socket: SocketRef,
    data: Value,
    server: Arc<Mutex<WarhorseServer<T>>>
) {

    info!(ns = socket.ns(), ?socket.id, "Socket.IO connected");
    socket.emit("hello", &crate::i18n::hello_message(Language::English)).ok();
    listen_for_user_login(&socket, server.clone());
    listen_for_user_registration(&socket, server.clone());
    listen_for_chat_messages(&socket, server.clone());
}
