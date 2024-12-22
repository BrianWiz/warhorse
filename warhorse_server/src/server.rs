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
        let user_partial = match req.identity {
            LoginUserIdentity::AccountName(account_name) => {
                self.data_service.users_get_by_account_name(&account_name)
            },
            LoginUserIdentity::Email(email) => {
                self.data_service.users_get_by_email(&email)
            },
        };

        if let Some(user) = user_partial {
            // @todo: do actual authentication here

            // register the user's socket
            self.user_sockets.insert(user.id, socket_id);
            Ok(())
        } else {
            Err(crate::i18n::invalid_login(req.language))?
        }
    }

    /// Registers a new user and logs them in if successful
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

    pub fn send_friend_request(&mut self, sender_id: UserId, req: FriendRequest) -> Result<(), ServerError> {
        if self.are_friends(sender_id.clone(), req.friend_id.clone()) {
            info!("{} is already friends with {}", sender_id, req.friend_id);
            Err(crate::i18n::already_friends(req.language))?;
        }

        let friend = self.data_service.users_get(req.friend_id.clone());
        if let Some(friend) = friend {
            self.data_service.friend_requests_insert(sender_id.clone(), req.friend_id.clone());
            let friend_socket_id = self.get_socket_id(req.friend_id.clone())?;
            if let Some(socket) = self.get_socket(friend_socket_id) {
                let friend = Friend {
                    id: friend.id.clone(),
                    display_name: friend.display_name.clone(),
                    status: self.get_online_status(friend.id.clone()),
                };
                let friend_request_accepted = FriendRequestAccepted { friend };
                let serialized_friend_request_accepted = friend_request_accepted.to_json()?;
                socket.emit("friend-request", &serialized_friend_request_accepted)?;
            }
        } else {
            Err(format!("{} does not exist", req.friend_id))?
        }

        Ok(())
    }

    pub fn accept_friend_request(&mut self, user_id: UserId, req: AcceptFriendRequest) -> Result<(), ServerError> {
        if self.are_friends(user_id.clone(), req.friend_id.clone()) {
            info!("{} is already friends with {}", user_id, req.friend_id);
            Err(crate::i18n::already_friends(req.language))?;
        }

        self.data_service.friends_add(user_id.clone(), req.friend_id.clone());
        let user_socket_id = self.get_socket_id(user_id.clone())?;
        if let Some(socket) = self.get_socket(user_socket_id) {
            let user = self.data_service.users_get(req.friend_id.clone());
            if let Some(user) = user {
                let friend = Friend {
                    id: user.id.clone(),
                    display_name: user.display_name.clone(),
                    status: self.get_online_status(user.id.clone()),
                };
                let friend_request_accepted = FriendRequestAccepted { friend };
                let serialized_friend_request_accepted = friend_request_accepted.to_json()?;
                socket.emit("friend-request-accepted", &serialized_friend_request_accepted)?;
            }
        }

        Ok(())
    }

    /// Rejects a friend request
    pub fn reject_friend_request(&mut self, user_id: UserId, req: RejectFriendRequest) -> Result<(), ServerError> {
        self.data_service.friend_requests_remove(user_id, req.friend_id);
        Ok(())
    }

    /// Removes a friend
    pub fn remove_friend(&mut self, user_id: UserId, req: RemoveFriendRequest) -> Result<(), ServerError> {
        self.data_service.friends_remove(user_id, req.friend_id);
        Ok(())
    }

    /// Blocks a user
    pub fn block_user(&mut self, user_id: UserId, req: BlockUserRequest) -> Result<(), ServerError> {
        self.data_service.user_blocks_insert(user_id, req.user_id);
        Ok(())
    }

    /// Unblocks a user
    pub fn unblock_user(&mut self, user_id: UserId, req: UnblockUserRequest) -> Result<(), ServerError> {
        self.data_service.user_blocks_remove(user_id, req.user_id);
        Ok(())
    }

    /// Whether two users are friends
    fn are_friends(&self, user_id: UserId, friend_id: UserId) -> bool {
        self.data_service.friends_get(user_id).iter().any(|f| f.id == friend_id)
    }

    /// Whether a user is in a specific room or not
    fn user_in_room(&self, user_id: UserId, room_id: RoomId) -> bool {
        let room_id_clone = room_id.clone();
        if self.room_exists(room_id) {
            match self.get_socket_id(user_id) {
                Ok(id) => {
                    match self.get_socket(id) {
                        Some(socket) => {
                            match socket.rooms() {
                                Ok(rooms) => {
                                    let room_id = room_id_clone.as_str();
                                    rooms.iter().any(|r| r == room_id)
                                }
                                Err(_) => false,
                            }
                        },
                        None => false,
                    }
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Whether a room exists or not
    fn room_exists(&self, room_id: RoomId) -> bool {
        let room_id = room_id.as_str();
        self.io.rooms().iter().flatten().any(|r| r == room_id)
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

pub fn listen_for_friend_requests<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on("friend-request", move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match FriendRequest::from_json(data) {
                Ok(data) => {
                    match server.lock().await.get_logged_in_user_id(socket.id) {
                        Some(sender_id) => {
                            if let Err(e) = server.lock().await.send_friend_request(sender_id, data) {
                                error!(ns = socket.ns(), ?socket.id, ?e, "Failed to send friend request");
                            }
                        },
                        None => {
                            error!(ns = socket.ns(), ?socket.id, "Failed to get user ID");
                        }
                    }
                }
                Err(e) => {
                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to parse friend request");
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
    _data: Value,
    server: Arc<Mutex<WarhorseServer<T>>>
) {

    info!(ns = socket.ns(), ?socket.id, "Socket.IO connected");
    socket.emit("hello", &crate::i18n::hello_message(Language::English)).ok();
    listen_for_user_login(&socket, server.clone());
    listen_for_user_registration(&socket, server.clone());
    listen_for_chat_messages(&socket, server.clone());
    listen_for_friend_requests(&socket, server.clone());
}
