use std::collections::HashMap;

use warhorse_protocol::{Friend, FriendStatus, UserPartial, UserId, UserRegistration};

use super::Database;

pub struct InMemoryDatabase {
    users: HashMap<UserId, UserPartial>,
    friendships: HashMap<UserId, Vec<UserId>>,
    friend_requests: HashMap<UserId, Vec<UserId>>,
    user_blocks: Vec<(UserId, UserId)>,
    next_user_id: usize,
}

impl Database for InMemoryDatabase {

    fn new(_connection_string: &str) -> Self {
        InMemoryDatabase {
            users: HashMap::new(),
            friendships: HashMap::new(),
            friend_requests: HashMap::new(),
            user_blocks: Vec::new(),
            next_user_id: 0,
        }
    }

    fn user_exists(&self, user_id: UserId) -> bool {
        self.users.contains_key(&user_id)
    }

    fn users_insert(&mut self, user: UserRegistration) -> UserId {
        let new_user_id = self.next_user_id.to_string();
        self.next_user_id += 1;
        let user = UserPartial {
            id: new_user_id.clone(),
            language: user.language,
            display_name_lower: user.display_name.to_lowercase(),
            display_name: user.display_name,
            account_name_lower: Some(user.account_name.to_lowercase()),
            account_name: Some(user.account_name),
            email: Some(user.email),
        };
        self.users.insert(new_user_id.clone(), user);
        new_user_id
    }

    fn users_get(&self, user_id: UserId) -> Option<UserPartial> {
        self.users.get(&user_id).cloned()
    }

    fn users_get_by_account_name(&self, account_name: &str) -> Option<UserPartial> {
        self.users.values().find(|user| {
            if let Some(user_account_name) = &user.account_name {
                user_account_name == account_name
            } else {
                false
            }
        }).cloned()
    }

    fn users_get_by_email(&self, email: &str) -> Option<UserPartial> {
        self.users.values().find(|user| {
            if let Some(user_email) = &user.email {
                user_email == email
            } else {
                false
            }
        }).cloned()
    }

    fn user_blocks_insert(&mut self, user_id: UserId, blocked_id: UserId) {
        self.user_blocks.push((user_id, blocked_id));
    }

    fn user_blocks_remove(&mut self, user_id: UserId, blocked_id: UserId) {
        self.user_blocks.retain(|(id, blocked)| id != &user_id || blocked != &blocked_id);
    }

    fn user_blocks_get_blocks_for_user(&self, user_id: UserId) -> Vec<Friend> {
        self.user_blocks.iter()
            .filter_map(|(id, blocked_id)| {
                if id == &user_id {
                    self.users_get(blocked_id.clone())
                } else {
                    None
                }
            })
            .map(|user| Friend {
                id: user.id,
                display_name: user.display_name,
                status: FriendStatus::Blocked,
            })
            .collect()
    }

    fn user_get_pending_friend_requests_for_user(&self, user_id: UserId) -> Vec<Friend> {
        self.friend_requests.iter()
            .filter_map(|(id, friend_requests)| {
                if friend_requests.contains(&user_id) {
                    self.users_get(id.clone())
                } else {
                    None
                }
            })
            .map(|user| Friend {
                id: user.id,
                display_name: user.display_name,
                status: FriendStatus::FriendRequestReceived,
            })
            .collect()
    }

    fn user_get_friend_request_invites_sent_for_user(&self, user_id: UserId) -> Vec<Friend> {
        // we need to do a deep search to find all the friend requests that the user has sent.
        self.friend_requests.iter()
            .filter_map(|(id, friend_requests)| {
                if id == &user_id {
                    Some(friend_requests)
                } else {
                    None
                }
            })
            .flat_map(|friend_requests| {
                friend_requests.iter()
                    .filter_map(|id| self.users_get(id.clone()))
                    .map(|user| Friend {
                        id: user.id,
                        display_name: user.display_name,
                        status: FriendStatus::FriendRequestSent,
                    })
                    .collect::<Vec<Friend>>()
            })
            .collect()
    }

    fn user_is_blocked(&self, user_id: UserId, blocked_id: UserId) -> bool {
        self.user_blocks.iter().any(|(id, blocked)| id == &user_id && blocked == &blocked_id)
    }

    fn friend_requests_insert(&mut self, user_id: UserId, friend_id: UserId) {
        if let Some(friend_requests) = self.friend_requests.get_mut(&user_id) {
            friend_requests.push(friend_id);
        } else {
            self.friend_requests.insert(user_id, vec![friend_id]);
        }
    }

    fn friend_requests_remove(&mut self, user_id: UserId, friend_id: UserId) {
        if let Some(friend_requests) = self.friend_requests.get_mut(&user_id) {
            friend_requests.retain(|id| id != &friend_id);
        }
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
                self.users_get(id.clone()).map(|user| Friend {
                    id: user.id.clone(),
                    display_name: user.display_name.clone(),
                    status: FriendStatus::Offline, // it is up to the caller to figure out the status, so we default to offline.
                })
            })
            .collect()
    }
}
