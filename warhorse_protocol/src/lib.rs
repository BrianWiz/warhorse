pub mod error;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::error::Error;

pub type UserId = String;
pub type RoomId = String;

pub trait ProtocolMessage: Send + Sync + Serialize {
    fn to_json(&self) -> Result<Value, Error> {
        serde_json::to_value(self)
            .map_err(|e| Error(e.to_string()))
    }

    fn from_json(json: Value) -> Result<Self, Error>
    where
        Self: Sized;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUser {
    pub account_name: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
}

impl ProtocolMessage for RegisterUser {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginResponse {
    Success(UserId),
    Failure(String),
}

impl ProtocolMessage for LoginResponse {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FriendStatus {
    Online,
    Offline,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Friend {
    pub id: String,
    pub display_name: String,
    pub status: FriendStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendsList {
    pub friends: Vec<Friend>,
}

impl From<Vec<Friend>> for FriendsList {
    fn from(friends: Vec<Friend>) -> Self {
        FriendsList { friends }
    }
}

impl ProtocolMessage for FriendsList {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ChatChannel {
    Room(RoomId),
    PrivateMessage(UserId),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendChatMessage {
    pub channel: ChatChannel,
    pub message: String,
}

impl ProtocolMessage for SendChatMessage {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub display_name: String,
    pub message: String,
    pub time: String,
}

impl ProtocolMessage for ChatMessage {
    fn from_json(json: Value) -> Result<Self, Error> {
        serde_json::from_value(json)
            .map_err(|e| Error(e.to_string()))
    }
}