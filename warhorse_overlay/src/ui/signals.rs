use std::{collections::HashMap, time::Instant};

use warhorse_client::warhorse_protocol::*;

#[derive(PartialEq, Eq)]
pub enum InteractiveState {
    Nothing,
    AddFriendModal,
    WhisperFriendModal(Friend),
    RemoveFriendModal(Friend),
    BlockFriendModal(Friend),
    UnblockFriendModal(Friend),
    AcceptFriendRequestModal(Friend),
    RejectFriendRequestModal(Friend),
    FriendContextMenu(String),
}

pub struct ReceivedHello(pub bool);

pub struct ReceivedLoggedIn(pub bool);

pub struct FriendsList(pub HashMap<FriendStatus, Vec<Friend>>);

pub struct ChatMessages(pub Vec<ChatMessage>);

#[derive(Clone, PartialEq)]
pub struct Notification {
    pub message: String,
    pub timestamp: Instant,
    pub notification_type: NotificationType,
}

#[derive(Clone, PartialEq)]
pub enum NotificationType {
    Generic,
    FriendRequestReceived,
    FriendAccepted,
}

pub struct Notifications(pub Vec<Notification>);
