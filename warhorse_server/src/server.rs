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
use tracing::log::warn;
use crate::data_access::DataAccess;
use crate::database::Database;
use crate::error::ServerError;
use crate::utils::{is_valid_email, validate_account_name, validate_display_name, validate_password};

type SocketId = Sid;

pub struct WarhorseServer<T>
where T: Database + Send + Sync + 'static
{
    data_service: DataAccess<T>,
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
            data_service: DataAccess::new(T::new(database_connection_string)),
        }
    }

    /// Gets the Socket.IO instance
    pub fn io(&self) -> &SocketIo {
        &self.io
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

            // Actually log them in
            self.user_sockets.insert(user.id.clone(), socket_id);

            // Send the user their friends list
            self.send_friend_list(user.id.clone());

            // Send the user their block list
            self.send_block_list(user.id);
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
        validate_password(&req.password, req.language)?;
        validate_account_name(&req.account_name, req.language)?;
        validate_display_name(&req.display_name, req.language)?;

        if !is_valid_email(&req.email) {
            return Err(crate::i18n::invalid_email(req.language));
        }

        if self.data_service.users_get_by_account_name(&req.account_name).is_some() {
            return Err(crate::i18n::account_name_already_exists(req.language));
        }

        if self.data_service.users_get_by_email(&req.email).is_some() {
            return Err(crate::i18n::email_already_exists(req.language));
        }

        // Log them in
        let new_user_id = self.data_service.users_insert(req);
        self.user_sockets.insert(new_user_id.clone(), socket_id);

        // Send the user their friends list
        self.send_friend_list(new_user_id.clone());

        // Send the user their block list
        self.send_block_list(new_user_id);

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

                    if self.data_service.user_is_blocked(sender_id.clone(), user_id.clone()) {
                        warn!("{} has blocked {} but is trying to send a private chat message", sender_id, user_id);
                        return Err(crate::i18n::user_is_blocked(message.language));
                    }

                    if self.data_service.user_is_blocked(user_id.clone(), sender_id.clone()) {
                        warn!("{} has blocked {} but {} is trying to send a private chat message", user_id, sender_id, sender_id);
                        return Err(crate::i18n::user_is_blocked(message.language));
                    }

                    let socket_id = self.get_socket_id(user_id.clone())?;
                    if let Some(socket) = self.get_socket(socket_id) {
                        socket.emit(EVENT_RECEIVE_CHAT_MESSAGE, &serialized_message)?;
                    } else {
                        Err(format!("{} is not connected", user_id))?;
                    }
                } else {
                    Err(format!("{} is not friends with {} but is trying to send a private chat message", sender_id, user_id))?;
                }
            },
            ChatChannel::Room(room_id) => {
                if self.user_in_room(sender_id.clone(), room_id.clone()) {
                    self.get_room(room_id)
                        .emit(EVENT_RECEIVE_CHAT_MESSAGE, &serialized_message)?;
                } else {
                    Err(format!("{} is not in room {}", sender_id, room_id))?;
                }
            }
        }

        Ok(())
    }

    pub fn send_friend_request(&mut self, sender_id: UserId, req: FriendRequest) -> Result<(), ServerError> {
        if self.are_friends(sender_id.clone(), req.friend_id.clone()) {
            warn!("{} is already friends with {} but is trying to send a friend request", sender_id, req.friend_id);
            return Err(crate::i18n::already_friends(req.language));
        }

        if self.data_service.user_is_blocked(sender_id.clone(), req.friend_id.clone()) {
            warn!("{} has blocked {} but is trying to send a friend request", sender_id, req.friend_id);
            return Err(crate::i18n::user_is_blocked(req.language));
        }

        if self.data_service.user_is_blocked(req.friend_id.clone(), sender_id.clone()) {
            warn!("{} has blocked {} but {} is trying to send a friend request", req.friend_id, sender_id, sender_id);
            return Err(crate::i18n::user_is_blocked(req.language));
        }

        if self.data_service.user_exists(req.friend_id.clone()) {
            self.data_service.friend_requests_insert(sender_id.clone(), req.friend_id.clone());
            let friend_socket_id = self.get_socket_id(req.friend_id.clone())?;
            if let Some(socket) = self.get_socket(friend_socket_id) {

                // Send the updated friend requests list to the user who received the friend request
                let friend_requests = self.data_service.friend_requests_get(req.friend_id.clone());
                socket.emit(EVENT_RECEIVE_FRIEND_REQUESTS, &vec_to_json(friend_requests)?)?;

                // Send the updated friends list to the user who sent the friend request,
                // because now they'll have friends with pending request as their status.
                self.send_friend_list(sender_id);
            }
        } else {
            Err(format!("{} does not exist", req.friend_id))?
        }

        Ok(())
    }

    pub fn accept_friend_request(&mut self, user_id: UserId, req: AcceptFriendRequest) -> Result<(), ServerError> {
        if self.are_friends(user_id.clone(), req.friend_id.clone()) {
            info!("{} is already friends with {}", user_id, req.friend_id);
            return Err(crate::i18n::already_friends(req.language));
        }

        if self.data_service.user_is_blocked(user_id.clone(), req.friend_id.clone()) {
            warn!("{} has blocked {} but is trying to accept a friend request", user_id, req.friend_id);
            return Err(crate::i18n::user_is_blocked(req.language));
        }

        if self.data_service.user_is_blocked(req.friend_id.clone(), user_id.clone()) {
            warn!("{} has blocked {} but {} is trying to accept a friend request", req.friend_id, user_id, user_id);
            return Err(crate::i18n::user_is_blocked(req.language));
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
                socket.emit(EVENT_RECEIVE_FRIEND_REQUEST_ACCEPTED, &serialized_friend_request_accepted)?;

                // refresh the friends list for both users
                self.send_friend_list(user_id);
                self.send_friend_list(req.friend_id);
            }
        }

        Ok(())
    }

    /// Rejects a friend request
    pub fn reject_friend_request(&mut self, user_id: UserId, req: RejectFriendRequest) -> Result<(), ServerError> {
        self.data_service.friend_requests_remove(user_id, req.friend_id.clone());

        // We send the friend list back to the user who was rejected so that they can see the updated status.
        self.send_friend_list(req.friend_id);
        Ok(())
    }

    /// Removes a friend
    pub fn remove_friend(&mut self, user_id: UserId, req: RemoveFriendRequest) -> Result<(), ServerError> {
        self.data_service.friends_remove(user_id.clone(), req.friend_id.clone());

        // We need to refresh both users friends list
        self.send_friend_list(user_id);
        self.send_friend_list(req.friend_id);
        Ok(())
    }

    /// Blocks a user
    pub fn block_user(&mut self, user_id: UserId, req: BlockUserRequest) -> Result<(), ServerError> {
        self.data_service.user_blocks_insert(user_id.clone(), req.user_id.clone());

        // We need to refresh both users friends list
        self.send_friend_list(user_id.clone());
        self.send_friend_list(req.user_id);

        // Refresh the block list for the user who blocked the other user
        self.send_block_list(user_id);
        Ok(())
    }

    /// Unblocks a user
    pub fn unblock_user(&mut self, user_id: UserId, req: UnblockUserRequest) -> Result<(), ServerError> {
        self.data_service.user_blocks_remove(user_id.clone(), req.user_id);
        self.send_block_list(user_id);
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

    fn send_friend_list(&self, user_id: UserId) {
        match vec_to_json(self.get_friends_list(user_id.clone())) {
            Ok(friends_list) => {
                match self.get_socket_id(user_id) {
                    Ok(socket_id) => {
                        if let Some(socket) = self.get_socket(socket_id) {
                            let _= socket.emit(EVENT_RECEIVE_FRIENDS, &friends_list);
                        }
                    },
                    Err(e) => {
                        error!(?e, "Failed to get socket ID");
                    }
                }
            },
            Err(e) => {
                error!(?e, "Failed to serialize friends list");
            }
        }
    }

    fn send_block_list(&self, user_id: UserId) {
        match vec_to_json(self.data_service.user_blocks_get_blocks_for_user(user_id.clone())) {
            Ok(blocks_list) => {
                match self.get_socket_id(user_id) {
                    Ok(socket_id) => {
                        if let Some(socket) = self.get_socket(socket_id) {
                            let _= socket.emit(EVENT_RECEIVE_BLOCKED_USERS, &blocks_list);
                        }
                    },
                    Err(e) => {
                        error!(?e, "Failed to get socket ID");
                    }
                }
            },
            Err(e) => {
                error!(?e, "Failed to serialize blocks list");
            }
        }
    }

    /// Whether a room exists or not
    fn room_exists(&self, room_id: RoomId) -> bool {
        let room_id = room_id.as_str();
        self.io.rooms().iter().flatten().any(|r| r == room_id)
    }

    /// Gets the user ID of the logged in user associated with a socket
    fn get_logged_in_user_id(&self, socket_id: SocketId) -> Option<UserId> {
        self.user_sockets.iter().find_map(|(user_id, id)| {
            if id == &socket_id {
                Some(user_id.clone())
            } else {
                None
            }
        })
    }

    /// Gets the friends list of a user and their online status
    fn get_friends_list(&self, user_id: UserId) -> Vec<Friend> {
        let mut friends_list = self.data_service.friends_get(user_id);
        for friend in friends_list.iter_mut() {
            // if they're not a pending friend request, we want to include their online status
            if matches!(friend.status, FriendStatus::PendingRequest) {
                friend.status = self.get_online_status(friend.id.clone());
            }
        }
        friends_list
    }
}

pub fn listen_for_chat_messages<T: Database + Send + Sync + 'static>(socket_ref: &SocketRef, server: Arc<Mutex<WarhorseServer<T>>>) {
    socket_ref.on(EVENT_SEND_CHAT_MESSAGE, move |socket: SocketRef, Data::<Value>(data)| {
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

pub fn listen_for_user_login<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_USER_LOGIN, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match UserLogin::from_json(data) {
                Ok(data) => {
                    match server.lock().await.login_user(data, socket.id).await {
                        Ok(_) => {
                            info!(ns = socket.ns(), ?socket.id, "User logged in");
                        },
                        Err(e) => {
                            error!(ns = socket.ns(), ?socket.id, ?e, "Failed to log in user");
                            match RequestError(e.0).to_json() {
                                Ok(json) => {
                                    match socket.emit(EVENT_RECEIVE_ERROR, &json) {
                                        Ok(_) => {
                                            info!(ns = socket.ns(), ?socket.id, "Sent error response");
                                        },
                                        Err(e) => {
                                            error!(ns = socket.ns(), ?socket.id, ?e, "Failed to send error response");
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to serialize error");
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to parse login data");
                }
            }
        }
    });
}

pub fn listen_for_user_registration<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_USER_REGISTER, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match UserRegistration::from_json(data) {
                Ok(data) => {
                    match server.lock().await.register_user(data, socket.id).await {
                        Ok(_) => {
                            info!(ns = socket.ns(), ?socket.id, "User registered");
                        },
                        Err(e) => {
                            info!(ns = socket.ns(), ?socket.id, ?e, "Failed to register user");
                            match RequestError(e.0).to_json() {
                                Ok(json) => {
                                    match socket.emit(EVENT_RECEIVE_ERROR, &json) {
                                        Ok(_) => {
                                            info!(ns = socket.ns(), ?socket.id, "Sent error response");
                                        },
                                        Err(e) => {
                                            error!(ns = socket.ns(), ?socket.id, ?e, "Failed to send error response");
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to serialize error");
                                }
                            }
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
    socket_ref.on(EVENT_SEND_FRIEND_REQUEST, move |socket: SocketRef, Data::<Value>(data)| {
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

pub fn listen_for_accept_friend_requests<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_FRIEND_REQUEST_ACCEPT, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match AcceptFriendRequest::from_json(data) {
                Ok(data) => {
                    match server.lock().await.get_logged_in_user_id(socket.id) {
                        Some(user_id) => {
                            if let Err(e) = server.lock().await.accept_friend_request(user_id, data) {
                                error!(ns = socket.ns(), ?socket.id, ?e, "Failed to accept friend request");
                            }
                        },
                        None => {
                            error!(ns = socket.ns(), ?socket.id, "Failed to get user ID");
                        }
                    }
                },
                Err(e) => {
                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to parse accept friend request");
                }
            }
        }
    });
}

pub fn listen_for_reject_friend_requests<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_FRIEND_REQUEST_REJECT, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match RejectFriendRequest::from_json(data) {
                Ok(data) => {
                    match server.lock().await.get_logged_in_user_id(socket.id) {
                        Some(user_id) => {
                            if let Err(e) = server.lock().await.reject_friend_request(user_id, data) {
                                error!(ns = socket.ns(), ?socket.id, ?e, "Failed to reject friend request");
                            }
                        },
                        None => {
                            error!(ns = socket.ns(), ?socket.id, "Failed to get user ID");
                        }
                    }
                },
                Err(e) => {
                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to parse reject friend request");
                }
            }
        }
    });
}

pub fn listen_for_remove_friend<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_FRIEND_REMOVE, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match RemoveFriendRequest::from_json(data) {
                Ok(data) => {
                    match server.lock().await.get_logged_in_user_id(socket.id) {
                        Some(user_id) => {
                            if let Err(e) = server.lock().await.remove_friend(user_id, data) {
                                error!(ns = socket.ns(), ?socket.id, ?e, "Failed to remove friend");
                            }
                        },
                        None => {
                            error!(ns = socket.ns(), ?socket.id, "Failed to get user ID");
                        }
                    }
                },
                Err(e) => {
                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to parse remove friend request");
                }
            }
        }
    });
}

pub fn listen_for_block_user_requests<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_USER_BLOCK, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match BlockUserRequest::from_json(data) {
                Ok(data) => {
                    match server.lock().await.get_logged_in_user_id(socket.id) {
                        Some(user_id) => {
                            if let Err(e) = server.lock().await.block_user(user_id, data) {
                                error!(ns = socket.ns(), ?socket.id, ?e, "Failed to block user");
                            }
                        },
                        None => {
                            error!(ns = socket.ns(), ?socket.id, "Failed to get user ID");
                        }
                    }
                },
                Err(e) => {
                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to parse block user request");
                }
            }
        }
    });
}

pub fn listen_for_unblock_user_requests<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_USER_UNBLOCK, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match UnblockUserRequest::from_json(data) {
                Ok(data) => {
                    match server.lock().await.get_logged_in_user_id(socket.id) {
                        Some(user_id) => {
                            if let Err(e) = server.lock().await.unblock_user(user_id, data) {
                                error!(ns = socket.ns(), ?socket.id, ?e, "Failed to unblock user");
                            }
                        },
                        None => {
                            error!(ns = socket.ns(), ?socket.id, "Failed to get user ID");
                        }
                    }
                },
                Err(e) => {
                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to parse unblock user request");
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
    listen_for_accept_friend_requests(&socket, server.clone());
    listen_for_reject_friend_requests(&socket, server.clone());
    listen_for_remove_friend(&socket, server.clone());
    listen_for_block_user_requests(&socket, server.clone());
    listen_for_unblock_user_requests(&socket, server.clone());
}
