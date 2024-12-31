use warhorse_protocol::{Friend, UserId, UserRegistration, UserPartial, FriendStatus};
use crate::database::Database;

/// DataAccess is a struct that provides a high-level interface to the database.
pub struct DataAccess<T>
    where T: Database
{
    database: T,
}

impl<T> DataAccess<T>
    where T: Database 
{
    pub fn new(database: T) -> Self {
        Self {
            database,
        }
    }

    pub fn user_exists(&self, user_id: UserId) -> bool {
        self.database.user_exists(user_id)
    }

    pub fn users_insert(&mut self, user: UserRegistration) -> UserId {
        self.database.users_insert(user)
    }

    pub fn friends_get(&self, user_id: UserId) -> Vec<Friend> {
        let friends = self.database.friends_get(user_id.clone());

        // we also want to include pending friend requests
        let friend_requests = self.database.friend_requests_get(user_id);

        // combine
        friends.iter().chain(friend_requests.iter()).cloned().collect()
    }

    pub fn friends_add(&mut self, user_id: UserId, friend_id: UserId) {
        self.database.friends_add(user_id, friend_id);
    }

    pub fn friends_remove(&mut self, user_id: UserId, friend_id: UserId) {
        self.database.friends_remove(user_id, friend_id);
    }

    pub fn friend_requests_insert(&mut self, user_id: UserId, friend_id: UserId) {
        self.database.friend_requests_insert(user_id, friend_id);
    }

    pub fn friend_requests_remove(&mut self, user_id: UserId, friend_id: UserId) {
        self.database.friend_requests_remove(user_id, friend_id);
    }

    pub fn friend_requests_get(&self, user_id: UserId) -> Vec<Friend> {
        let mut friend_requests = self.database.friend_requests_get(user_id);
        friend_requests.iter_mut().for_each(|friend| {
            friend.status = FriendStatus::InviteSent;
        });
        friend_requests
    }

    pub fn users_get(&self, user_id: UserId) -> Option<UserPartial> {
        self.database.users_get(user_id)
    }

    pub fn users_get_by_account_name(&self, account_name: &str) -> Option<UserPartial> {
        self.database.users_get_by_account_name(account_name)
    }

    pub fn users_get_by_email(&self, email: &str) -> Option<UserPartial> {
        self.database.users_get_by_email(email)
    }

    pub fn user_blocks_insert(&mut self, user_id: UserId, blocked_id: UserId) {
        self.database.user_blocks_insert(user_id.clone(), blocked_id.clone());
        self.friends_remove(user_id.clone(), blocked_id.clone());
        self.friends_remove(blocked_id.clone(), user_id.clone());
        self.friend_requests_remove(user_id.clone(), blocked_id.clone());
        self.friend_requests_remove(user_id.clone(), blocked_id.clone());
    }

    pub fn user_blocks_remove(&mut self, user_id: UserId, blocked_id: UserId) {
        self.database.user_blocks_remove(user_id, blocked_id);
    }

    pub fn user_blocks_get_blocks_for_user(&self, user_id: UserId) -> Vec<UserPartial> {
        self.database.user_blocks_get_blocks_for_user(user_id)
    }

    pub fn user_is_blocked(&self, user_id: UserId, blocked_id: UserId) -> bool {
        self.database.user_is_blocked(user_id, blocked_id)
    }
}
