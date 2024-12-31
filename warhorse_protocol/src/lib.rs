pub mod error;

use std::collections::HashMap;
use std::hash::Hash;
use std::path::Display;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;
use crate::error::Error;

pub type UserId = String;
pub type RoomId = String;

// For validation on both backend and frontend
pub const ACCOUNT_NAME_MAX_LENGTH: usize = 20;
pub const ACCOUNT_NAME_MIN_LENGTH: usize = 3;
pub const DISPLAY_NAME_MAX_LENGTH: usize = 20;
pub const DISPLAY_NAME_MIN_LENGTH: usize = 3;
pub const PASSWORD_MIN_LENGTH: usize = 8;

// Socket.IO Events, named from the client's perspective.

/// Event for getting connection approval from the server.
pub const EVENT_RECEIVE_HELLO: &str = "hello";

/// Event for sending a user login to the server.
pub const EVENT_SEND_USER_LOGIN: &str = "/user/login";

/// Event for sending a user register to the server.
pub const EVENT_SEND_USER_REGISTER: &str = "/user/register";

/// Event for sending a user logout to the server.
pub const EVENT_SEND_USER_LOGOUT: &str = "/user/logout";

/// Event for sending a user block to the server.
pub const EVENT_SEND_USER_BLOCK: &str = "/user/block";

/// Event for sending a user unblock to the server.
pub const EVENT_SEND_USER_UNBLOCK: &str = "/user/unblock";

/// Event for sending a friend request to the server.
pub const EVENT_SEND_FRIEND_REQUEST: &str = "/friend/request";

/// Event for sending a friend request accept to the server.
pub const EVENT_SEND_FRIEND_REQUEST_ACCEPT: &str = "/friend/request/accept";

/// Event for sending a friend request reject to the server.
pub const EVENT_SEND_FRIEND_REQUEST_REJECT: &str = "/friend/request/reject";

/// Event for sending a friend remove to the server.
pub const EVENT_SEND_FRIEND_REMOVE: &str = "/friend/remove";

/// Event for sending a chat message to the server.
pub const EVENT_SEND_CHAT_MESSAGE: &str = "/chat/send";

/// Event for receiving a successful user login response, received from the server.
pub const EVENT_RECEIVE_USER_LOGIN: &str = "/user/login";

/// Event for receiving an error response, received from the server.
pub const EVENT_RECEIVE_ERROR: &str = "/error";

/// Event for receiving your friend list, received from the server.
pub const EVENT_RECEIVE_FRIENDS: &str = "/friends/receive";

/// Event for receiving a blocked list of users, received from the server.
pub const EVENT_RECEIVE_BLOCKED_USERS: &str = "/blocked_users/receive";

/// Event for receiving a friend request, invoked by a user, but ultimately received from the server.
pub const EVENT_RECEIVE_FRIEND_REQUESTS: &str = "/friend_requests/receive";

/// Event for receiving a friend request response, received from the server.
pub const EVENT_RECEIVE_FRIEND_REQUEST_ACCEPTED: &str = "/friend_request/accepted";

/// Event for receiving a chat message, invoked by a user, but ultimately received from the server.
pub const EVENT_RECEIVE_CHAT_MESSAGE: &str = "/chat/receive";

/// Base trait for all protocol types.
pub trait ProtoType: Send + Sync + Serialize + DeserializeOwned {
    fn to_json(&self) -> Result<Value, Error> {
        serde_json::to_value(self)
            .map_err(|e| Error(e.to_string()))
    }

    fn from_json(json: Value) -> Result<Self, Error>
    where
        Self: Sized {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

/// Serialize a vector of messages to JSON.
pub fn vec_to_json<T: ProtoType>(messages: Vec<T>) -> Result<Value, Error> {
    serde_json::to_value(messages)
        .map_err(|e| Error(e.to_string()))
}

/// Deserialize a vector of messages from JSON.
pub fn json_to_vec<T: ProtoType>(json: Value) -> Result<Vec<T>, Error> {
    serde_json::from_value(json)
        .map_err(|e| Error(e.to_string()))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Language {
    English,
    Spanish,
    French,
}

impl ProtoType for Language {}


/// Represents a user in the system, but with sensitive information removed.
/// And options to reduce the amount of data/sensitive info sent depending on the context.
/// Regardless, we never include the password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPartial {
    pub id: UserId,
    pub display_name_lower: String,
    pub display_name: String,
    pub account_name_lower: Option<String>,
    pub account_name: Option<String>,
    pub email: Option<String>,
    pub language: Language,
}

impl ProtoType for UserPartial {}

/// A user may login with either their account name or email
#[derive(Debug, Serialize, Deserialize)]
pub enum LoginUserIdentity {
    AccountName(String),
    Email(String),
}

impl ProtoType for LoginUserIdentity {}

/// Request to login a user
#[derive(Debug, Serialize, Deserialize)]
pub struct UserLogin {
    pub language: Language,
    pub identity: LoginUserIdentity,
    pub password: String,
}

impl ProtoType for UserLogin {}

/// Request to register a new user
#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegistration {
    pub language: Language,
    pub account_name: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
}

impl ProtoType for UserRegistration {}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestError(pub String);

impl ProtoType for RequestError {}

/// The online status of a friend
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum FriendStatus {
    Online,
    Offline,
    InviteSent,
    PendingRequest,
    Blocked,
}

impl std::fmt::Display for FriendStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FriendStatus::Online => write!(f, "Online"),
            FriendStatus::Offline => write!(f, "Offline"),
            FriendStatus::InviteSent => write!(f, "Invite Sent"),
            FriendStatus::PendingRequest => write!(f, "Pending Request"),
            FriendStatus::Blocked => write!(f, "Blocked"),
        }
    }
}

impl Hash for FriendStatus {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

/// A friend of a user
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Friend {
    pub id: String,
    pub display_name: String,
    pub status: FriendStatus,
}

impl ProtoType for Friend {}

/// A friend request
#[derive(Debug, Serialize, Deserialize)]
pub struct FriendRequest {
    pub language: Language,
    pub friend_id: UserId,
}

impl ProtoType for FriendRequest {}

/// Accept a friend request
#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptFriendRequest {
    pub language: Language,
    pub friend_id: UserId,
}

impl ProtoType for AcceptFriendRequest {}

/// Reject a friend request
#[derive(Debug, Serialize, Deserialize)]
pub struct RejectFriendRequest {
    pub language: Language,
    pub friend_id: UserId,
}

impl ProtoType for RejectFriendRequest {}

/// A friend request response
#[derive(Debug, Serialize, Deserialize)]
pub struct FriendRequestAccepted {
    pub friend: Friend,
}

impl ProtoType for FriendRequestAccepted {}

/// Request to remove a friend.
#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveFriendRequest {
    pub language: Language,
    pub friend_id: UserId,
}

impl ProtoType for RemoveFriendRequest {}

/// Request to block a user.
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockUserRequest {
    pub language: Language,
    pub user_id: UserId,
}

impl ProtoType for BlockUserRequest {}

/// Request to unblock a user.
#[derive(Debug, Serialize, Deserialize)]
pub struct UnblockUserRequest {
    pub language: Language,
    pub user_id: UserId,
}

impl ProtoType for UnblockUserRequest {}

/// A chat channel can either be a room or a private message to another user.
#[derive(Debug, Serialize, Deserialize)]
pub enum ChatChannel {
    Room(RoomId),
    PrivateMessage(UserId),
}

impl ProtoType for ChatChannel {}

/// Request to send a chat message.
#[derive(Debug, Serialize, Deserialize)]
pub struct SendChatMessage {
    pub language: Language,
    pub channel: ChatChannel,
    pub message: String,
}

impl ProtoType for SendChatMessage {}

/// A chat message.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub display_name: String,
    pub message: String,
    pub time: u32,
}

impl ProtoType for ChatMessage {}

pub fn categorize_friends(friends: Vec<Friend>) -> HashMap<FriendStatus, Vec<Friend>> {
    let mut categorized = HashMap::new();
    for friend in friends {
        let status = friend.status;
        let list = categorized.entry(status).or_insert_with(Vec::new);
        list.push(friend);
    }
    categorized
}
