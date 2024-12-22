pub mod error;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::error::Error;

pub type UserId = String;
pub type RoomId = String;

/// Base trait for all protocol types.
pub trait ProtoType: Send + Sync + Serialize {
    fn to_json(&self) -> Result<Value, Error> {
        serde_json::to_value(self)
            .map_err(|e| Error(e.to_string()))
    }

    fn from_json(json: Value) -> Result<Self, Error>
    where
        Self: Sized;
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

impl ProtoType for Language {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

/// Represents a user in the system, but with sensitive information removed.
/// And options to reduce the amount of data/sensitive info sent depending on the context.
/// Regardless, we never include the password.
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

impl ProtoType for UserPartial {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

/// A user may login with either their account name or email.
#[derive(Debug, Serialize, Deserialize)]
pub enum LoginUserIdentity {
    AccountName(String),
    Email(String),
}

impl ProtoType for LoginUserIdentity {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

/// Request to login a user.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserLogin {
    pub language: Language,
    pub identity: LoginUserIdentity,
    pub password: String,
}

impl ProtoType for UserLogin {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

/// Request to register a new user.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegistration {
    pub language: Language,
    pub account_name: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
}

impl ProtoType for UserRegistration {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

/// Response to a login or registration request (registering automatically logs in the user).
#[derive(Debug, Serialize, Deserialize)]
pub enum LoginResponse {
    Success(UserId),
    Failure(String),
}

impl ProtoType for LoginResponse {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

/// The online status of a friend.
#[derive(Debug, Serialize, Deserialize)]
pub enum FriendOnlineStatus {
    Online,
    Offline,
}

/// A friend of a user.
#[derive(Debug, Serialize, Deserialize)]
pub struct Friend {
    pub id: String,
    pub display_name: String,
    pub status: FriendOnlineStatus,
}

impl ProtoType for Friend {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
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

impl ProtoType for ChatMessage {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

