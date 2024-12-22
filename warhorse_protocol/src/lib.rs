pub mod error;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;
use crate::error::Error;

pub type UserId = String;
pub type RoomId = String;

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

/// Response to a login or registration request (registering automatically logs in the user)
#[derive(Debug, Serialize, Deserialize)]
pub enum LoginResponse {
    Success(UserId),
    Failure(String),
}

impl ProtoType for LoginResponse {}

/// The online status of a friend
#[derive(Debug, Serialize, Deserialize)]
pub enum FriendOnlineStatus {
    Online,
    Offline,
}

/// A friend of a user
#[derive(Debug, Serialize, Deserialize)]
pub struct Friend {
    pub id: String,
    pub display_name: String,
    pub status: FriendOnlineStatus,
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
pub struct AcceptFriendRequest {
    pub language: Language,
    pub friend_id: UserId,
}

/// Reject a friend request
pub struct RejectFriendRequest {
    pub language: Language,
    pub friend_id: UserId,
}

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

/// A chat channel can either be a room or a private message to another user.
#[derive(Debug, Serialize, Deserialize)]
pub enum ChatChannel {
    Room(RoomId),
    PrivateMessage(UserId),
}

/// Request to send a chat message.
#[derive(Debug, Serialize, Deserialize)]
pub struct SendChatMessage {
    pub channel: ChatChannel,
    pub message: String,
}

impl ProtoType for SendChatMessage {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

/// A chat message.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub display_name: String,
    pub message: String,
    pub time: String,
}

impl ProtoType for ChatMessage {}

