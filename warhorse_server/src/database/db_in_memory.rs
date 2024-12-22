use std::collections::HashMap;

use horse_protocol::{Friend, FriendStatus, UserId};

use crate::data_service::User;

use super::Database;

pub struct InMemoryDatabase {
    users: HashMap<UserId, User>,
    friendships: HashMap<UserId, Vec<UserId>>,
}

impl Database for InMemoryDatabase {

    fn new() -> Self {
        InMemoryDatabase {
            users: HashMap::new(),
            friendships: HashMap::new(),
        }
    }

    fn user_insert(&mut self, user: User) {
        self.users.insert(user.id.clone(), user);
    }

    fn user_get(&self, user_id: UserId) -> Option<User> {
        self.users.get(&user_id).cloned()
    }

    fn user_get_by_account_name(&self, account_name: &str) -> Option<User> {
        self.users.values().find(|user| user.account_name == account_name).cloned()
    }

    fn user_get_by_email(&self, email: &str) -> Option<User> {
        self.users.values().find(|user| user.email == email).cloned()
    }

    fn friends_add(&mut self, user_id: UserId, friend_id: UserId) {
        if let Some(friends) = self.friendships.get_mut(&user_id) {
            friends.push(friend_id);
        } else {
            self.friendships.insert(user_id, vec![friend_id]);
        }
    }

    fn friends_remove(&mut self, user_id: UserId, friend_id: UserId) {
        if let Some(friends) = self.friendships.get_mut(&user_id) {
            friends.retain(|id| id != &friend_id);
        }
    }

    fn friends_get(&self, user_id: UserId) -> Vec<Friend> {
        self.friendships.get(&user_id).cloned().unwrap_or_default()
            .iter()
            .filter_map(|id| {
                self.user_get(id.clone()).map(|user| Friend {
                    id: user.id.clone(),
                    display_name: user.display_name.clone(),
                    status: FriendStatus::Offline, // it is up to the caller to figure out the status, so we default to offline.
                })
            })
            .collect()
    }
}
