use std::{sync::Arc, time::Instant};
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
            self.send_post_login_data(user.id);
            Ok(())
        } else {
            Err(crate::i18n::invalid_login(req.language))?
        }
    }

    /// Registers a new user and logs them in if successful
    pub async fn register_user(
        &mut self,
        req: UserRegistration,
        socket_id: Option<SocketId>
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

        // insert into the db
        let new_user_id = self.data_service.users_insert(req);
        info!("Registered new user: {}", new_user_id);

        // log them in if there's a socket available
        if let Some(socket_id) = socket_id {
            self.user_sockets.insert(new_user_id.clone(), socket_id);
            self.send_post_login_data(new_user_id);
        }
        Ok(())
    }

    /// Removes a user's socket
    pub async fn remove_user(&mut self, user_id: &str) {
        self.user_sockets.remove(user_id);
    }

    /// Sends post login data to the user
    fn send_post_login_data(&self, user_id: UserId) {
        self.send_friend_list(user_id.clone());
        self.send_friend_requests(user_id.clone());
        self.send_post_login_event(user_id);
    }

    /// Sends a post login event
    fn send_post_login_event(&self, user_id: UserId) {
        match self.get_socket_id(user_id) {
            Ok(socket_id) => {
                if let Some(socket) = self.get_socket(socket_id) {
                    let _= socket.emit(EVENT_RECEIVE_USER_LOGIN, &serde_json::json!({}));
                }
            },
            Err(e) => {
                info!(?e, "Failed to get socket ID");
            }
        }
    }

    /// Sends a private message to a specific user
    fn send_chat_message(&self, sender_id: UserId, message: SendChatMessage) -> Result<(), ServerError> {

        let display_name = match self.data_service.users_get(sender_id.clone()) {
            Some(user) => user.display_name.clone(),
            None => {
                error!("User does not exist: {}", sender_id);
                return Err(format!("{} does not exist", sender_id))?;
            }
        };

        let serialized_message = ChatMessage {
            display_name,
            channel: message.channel.clone(),
            message: message.message.clone(),
            time: chrono::Utc::now().timestamp() as u32,
        }.to_json()?;

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

    fn send_friend_requests(&self, user_id: UserId) {
        match vec_to_json(self.data_service.user_get_pending_friend_requests_for_user(user_id.clone())) {
            Ok(friend_requests) => {
                match self.get_socket_id(user_id) {
                    Ok(socket_id) => {
                        if let Some(socket) = self.get_socket(socket_id) {
                            let _= socket.emit(EVENT_RECEIVE_FRIEND_REQUESTS, &friend_requests);
                        }
                    },
                    Err(e) => {
                        info!(?e, "Failed to get socket ID");
                    }
                }
            },
            Err(e) => {
                error!(?e, "Failed to serialize friend requests");
            }
        }
    }

    fn send_friend_request(&mut self, sender_id: UserId, req: FriendRequest) -> Result<(), ServerError> {
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

            // send a friend request to the target user
            self.send_friend_requests(req.friend_id.clone());

            // refresh the friends list for the sender
            self.send_friend_list(sender_id);

            // refresh the friends list for the target user
            self.send_friend_list(req.friend_id);
        } else {
            error!("User does not exist: {}", req.friend_id);
            return Err(format!("{} does not exist", req.friend_id))?;
        }

        Ok(())
    }

    fn accept_friend_request(&mut self, user_id: UserId, req: AcceptFriendRequest) -> Result<(), ServerError> {
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
    fn reject_friend_request(&mut self, user_id: UserId, req: RejectFriendRequest) -> Result<(), ServerError> {
        self.data_service.friend_requests_remove(user_id.clone(), req.friend_id.clone());

        // refresh the friends list for both users
        self.send_friend_list(req.friend_id);
        self.send_friend_list(user_id);
        Ok(())
    }

    /// Removes a friend
    fn remove_friend(&mut self, user_id: UserId, req: RemoveFriendRequest) -> Result<(), ServerError> {
        info!("Removing friend: {:?}", req);
        self.data_service.friends_remove(user_id.clone(), req.friend_id.clone());

        // We need to refresh both users friends list
        self.send_friend_list(user_id);
        self.send_friend_list(req.friend_id);
        Ok(())
    }

    /// Blocks a user
    fn block_user(&mut self, user_id: UserId, req: BlockUserRequest) -> Result<(), ServerError> {
        self.data_service.friends_remove(user_id.clone(), req.user_id.clone());
        self.data_service.user_blocks_insert(user_id.clone(), req.user_id.clone());

        // We need to refresh both users friends list
        self.send_friend_list(user_id.clone());
        self.send_friend_list(req.user_id);
        Ok(())
    }

    /// Unblocks a user
    fn unblock_user(&mut self, user_id: UserId, req: UnblockUserRequest) -> Result<(), ServerError> {
        self.data_service.user_blocks_remove(user_id.clone(), req.user_id.clone());
        
        // We need to refresh both users friends list
        self.send_friend_list(user_id.clone());
        self.send_friend_list(req.user_id);
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
                        info!(?e, "Failed to get socket ID");
                    }
                }
            },
            Err(e) => {
                error!(?e, "Failed to serialize friends list");
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
            // get their online status if they are a friend who:
            // - is not a pending friend request
            // - is not a friend request invite sent
            // - is not blocked
            if friend.status == FriendStatus::Offline {
                friend.status = self.get_online_status(friend.id.clone());
            }
        }
        friends_list
    }
}

fn listen_for_chat_messages<T: Database + Send + Sync + 'static>(socket_ref: &SocketRef, server: Arc<Mutex<WarhorseServer<T>>>) {
    socket_ref.on(EVENT_SEND_CHAT_MESSAGE, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match SendChatMessage::from_json(data) {
                Ok(data) => {
                    let logged_in_user_id = server.lock().await.get_logged_in_user_id(socket.id);
                    if let Some(logged_in_user_id) = logged_in_user_id {
                        if let Err(e) = server.lock().await.send_chat_message(logged_in_user_id, data) {
                            info!(ns = socket.ns(), ?socket.id, ?e, "Failed to send chat message");
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

fn listen_for_user_login<T: Database + Send + Sync + 'static>(
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
                            info!(ns = socket.ns(), ?socket.id, ?e, "Failed to log in user");
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

fn listen_for_user_registration<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_USER_REGISTER, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match UserRegistration::from_json(data) {
                Ok(data) => {
                    match server.lock().await.register_user(data, Some(socket.id)).await {
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

fn listen_for_friend_requests<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    info!("Setting up friend request listener");
    socket_ref.on(EVENT_SEND_FRIEND_REQUEST, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            info!("Received friend request data: {:?}", data);
            match FriendRequest::from_json(data) {
                Ok(data) => {
                    info!("Parsed friend request: {:?}", data);
                    let mut server = server.lock().await;
                    match server.get_logged_in_user_id(socket.id) {
                        Some(sender_id) => {
                            info!("Found sender ID: {}", sender_id);

                            if sender_id == data.friend_id {
                                info!(ns = socket.ns(), ?socket.id, "User tried to send a friend request to themselves, it was ignored");
                                return;
                            }

                            if let Err(e) = server.send_friend_request(sender_id, data) {
                                info!(ns = socket.ns(), ?socket.id, ?e, "Failed to send friend request");
                            } else {
                                info!("Friend request processed successfully");
                            }
                        },
                        None => {
                            info!(ns = socket.ns(), ?socket.id, "Failed to get user ID - user might not be logged in");
                        }
                    }
                }
                Err(e) => {
                    error!(ns = socket.ns(), ?socket.id, ?e, "Failed to parse friend request data");
                }
            }
        }
    });
}

fn listen_for_accept_friend_requests<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_FRIEND_REQUEST_ACCEPT, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match AcceptFriendRequest::from_json(data) {
                Ok(data) => {
                    let logged_in_user_id = server.lock().await.get_logged_in_user_id(socket.id);
                    match logged_in_user_id {
                        Some(user_id) => {
                            if let Err(e) = server.lock().await.accept_friend_request(user_id, data) {
                                info!(ns = socket.ns(), ?socket.id, ?e, "Failed to accept friend request");
                            }
                        },
                        None => {
                            info!(ns = socket.ns(), ?socket.id, "Failed to get user ID - user might not be logged in");
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

fn listen_for_reject_friend_requests<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_FRIEND_REQUEST_REJECT, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match RejectFriendRequest::from_json(data) {
                Ok(data) => {
                    let logged_in_user_id = server.lock().await.get_logged_in_user_id(socket.id);
                    match logged_in_user_id {
                        Some(user_id) => {
                            if let Err(e) = server.lock().await.reject_friend_request(user_id, data) {
                                info!(ns = socket.ns(), ?socket.id, ?e, "Failed to reject friend request");
                            }
                        },
                        None => {
                            info!(ns = socket.ns(), ?socket.id, "Failed to get user ID - user might not be logged in");
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

fn listen_for_remove_friend<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_FRIEND_REMOVE, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match RemoveFriendRequest::from_json(data) {
                Ok(data) => {
                    let logged_in_user_id = server.lock().await.get_logged_in_user_id(socket.id);
                    match logged_in_user_id {
                        Some(user_id) => {
                            if let Err(e) = server.lock().await.remove_friend(user_id, data) {
                                info!(ns = socket.ns(), ?socket.id, ?e, "Failed to remove friend");
                            }
                        },
                        None => {
                            info!(ns = socket.ns(), ?socket.id, "Failed to get user ID - user might not be logged in");
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

fn listen_for_block_user_requests<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_USER_BLOCK, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match BlockUserRequest::from_json(data) {
                Ok(data) => {
                    let logged_in_user_id = server.lock().await.get_logged_in_user_id(socket.id);
                    match logged_in_user_id {
                        Some(user_id) => {
                            if let Err(e) = server.lock().await.block_user(user_id, data) {
                                info!(ns = socket.ns(), ?socket.id, ?e, "Failed to block user");
                            }
                        },
                        None => {
                            info!(ns = socket.ns(), ?socket.id, "Failed to get user ID - user might not be logged in");
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

fn listen_for_unblock_user_requests<T: Database + Send + Sync + 'static>(
    socket_ref: &SocketRef,
    server: Arc<Mutex<WarhorseServer<T>>>
) {
    socket_ref.on(EVENT_SEND_USER_UNBLOCK, move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            match UnblockUserRequest::from_json(data) {
                Ok(data) => {
                    let logged_in_user_id = server.lock().await.get_logged_in_user_id(socket.id);
                    match logged_in_user_id {
                        Some(user_id) => {
                            if let Err(e) = server.lock().await.unblock_user(user_id, data) {
                                info!(ns = socket.ns(), ?socket.id, ?e, "Failed to unblock user");
                            }
                        },
                        None => {
                            info!(ns = socket.ns(), ?socket.id, "Failed to get user ID - user might not be logged in");
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

fn handle_user_disconnect<T: Database + Send + Sync + 'static>(
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

    socket.emit(EVENT_RECEIVE_HELLO, &crate::i18n::hello_message(Language::English)).ok();

    // add them to the general chat room, everyone is in general
    socket.join("general").ok();

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
