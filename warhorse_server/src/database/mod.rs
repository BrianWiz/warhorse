use warhorse_protocol::{Friend, UserPartial, UserId, UserRegistration};

pub mod db_in_memory;
pub mod db_postgres;

pub trait Database {
    fn new(connection_string: &str) -> Self;

    // Users
    fn users_insert(&mut self, user: UserRegistration) -> UserId;
    fn users_get(&self, user_id: UserId) -> Option<UserPartial>;
    fn users_get_by_account_name(&self, account_name: &str) -> Option<UserPartial>;
    fn users_get_by_email(&self, email: &str) -> Option<UserPartial>;

    // Friends
    fn friend_requests_add(&mut self, user_id: UserId, friend_id: UserId);
    fn friend_requests_remove(&mut self, user_id: UserId, friend_id: UserId);
    fn friends_add(&mut self, user_id: UserId, friend_id: UserId);
    fn friends_remove(&mut self, user_id: UserId, friend_id: UserId);
    fn friends_get(&self, user_id: UserId) -> Vec<Friend>;
}