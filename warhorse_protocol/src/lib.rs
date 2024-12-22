use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type UserId = String;
pub type RoomId = String;

//==================================================================================================
// AUTH
//==================================================================================================
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUser {
    pub account_name: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginResponse {
    Success(UserId),
    Failure(String),
}

impl LoginResponse {
    pub fn to_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    pub fn from_json(json: Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(json)
    }
}

//==================================================================================================
// FRIENDS
//==================================================================================================

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
    friends: Vec<Friend>,
}

impl From<Vec<Friend>> for FriendsList {
    fn from(friends: Vec<Friend>) -> Self {
        FriendsList { friends }
    }
}

impl FriendsList {
    pub fn to_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    pub fn from_json(json: Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(json)
    }
}

//==================================================================================================
// CHAT
//==================================================================================================

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

impl SendChatMessage {
    pub fn to_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    pub fn from_json(json: Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(json)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub display_name: String,
    pub message: String,
    pub time: String,
}

impl ChatMessage {
    pub fn to_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    pub fn from_json(json: Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(json)
    }
}
