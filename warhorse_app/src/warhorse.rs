use std::collections::HashMap;
use dioxus::logger::tracing::info;
use warhorse_client::{warhorse_protocol::*, WarhorseClient, WarhorseEvent};

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
    FriendContextMenu(String)
}

pub struct Warhorse {
    pub client: Option<WarhorseClient>,
}

impl Warhorse {
    pub fn send_friend_request(&mut self, id: String) {
        if let Some(client) = &self.client {
            if let Ok(()) = client.send_friend_request(&id) {
                info!("Sent friend request to {}", id);
            }
        }
    }

    pub fn send_user_login_request(&mut self, username: String, password: String) {
        if let Some(client) = &self.client {
            let username_clone = username.clone();
            let user_login_request = UserLogin {
                language: Language::English,
                identity: if Self::is_email_as_username(&username) {
                    LoginUserIdentity::Email(username)
                } else {
                    LoginUserIdentity::AccountName(username)
                },
                password,
            };
            if let Ok(()) = client.send_user_login_request(user_login_request) {
                info!("Sent login request for user {}", username_clone);
            }
        }
    }
    
    pub fn send_user_registration_request(&mut self, account_name: String, password: String, display_name: String, email: String) {
        if let Some(client) = &self.client {
            let account_name_clone = account_name.clone();
            let user_registration_request = UserRegistration {
                account_name,
                password,
                email,
                display_name,
                language: Language::English,
            };
            if let Ok(()) = client.send_user_registration_request(user_registration_request) {
                info!("Sent registration request for user {}", account_name_clone);
            }
        }
    }

    pub fn send_whisper_message(&mut self, friend_id: String, message: String) {
        if let Some(client) = &self.client {
            let message = SendChatMessage {
                language: Language::English,
                message,
                channel: ChatChannel::PrivateMessage(friend_id.clone()),
            };
            if let Ok(()) = client.send_chat_message(message) {
                info!("Sent whisper message to {}", friend_id);
            }
        }
    }

    pub fn send_chat_message(&mut self, message: String) {
        if let Some(client) = &self.client {
            let message = SendChatMessage {
                language: Language::English,
                message,
                channel: ChatChannel::Room("general".to_string()),
            };
            if let Ok(()) = client.send_chat_message(message) {
                info!("Sent chat message to #general");
            }
        }
    }

    pub fn send_block_friend(&mut self, friend_id: String) {
        if let Some(client) = &self.client {
            if let Ok(()) = client.send_block_friend(&friend_id) {
                info!("Sent request to block friend {}", friend_id);
            }
        }
    }

    pub fn send_unblock_friend(&mut self, friend_id: String) {
        if let Some(client) = &self.client {
            if let Ok(()) = client.send_unblock_friend(&friend_id) {
                info!("Sent request to unblock friend {}", friend_id);
            }
        }
    }

    pub fn send_remove_friend(&mut self, friend_id: String) {
        if let Some(client) = &self.client {
            if let Ok(()) = client.send_remove_friend(&friend_id) {
                info!("Sent request to remove friend {}", friend_id);
            }
        }
    }

    pub fn send_accept_friend_request(&mut self, friend_id: String) {
        if let Some(client) = &self.client {
            if let Ok(()) = client.send_accept_friend_request(&friend_id) {
                info!("Sent request to accept friend request from {}", friend_id);
            }
        }
    }

    pub fn send_reject_friend_request(&mut self, friend_id: String) {
        if let Some(client) = &self.client {
            if let Ok(()) = client.send_reject_friend_request(&friend_id) {
                info!("Sent request to reject friend request from {}", friend_id);
            }
        }
    }

    pub fn pump(&mut self) -> Vec<WarhorseEvent> {
        if let Some(client) = &self.client {
            client.pump()
        } else {
            vec![]
        }
    }

    fn is_email_as_username(input: &str) -> bool {
        input.contains('@')
    }
}
