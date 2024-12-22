use warhorse_protocol::{Friend, UserId};

use crate::data_service::User;

pub mod db_in_memory;
pub mod db_postgres;

pub trait Database {
    fn new() -> Self;
    fn user_insert(&mut self, user: User);
    fn user_get(&self, user_id: UserId) -> Option<User>;
    fn user_get_by_account_name(&self, account_name: &str) -> Option<User>;
    fn user_get_by_email(&self, email: &str) -> Option<User>;
    fn friends_add(&mut self, user_id: UserId, friend_id: UserId);
    fn friends_remove(&mut self, user_id: UserId, friend_id: UserId);
    fn friends_get(&self, user_id: UserId) -> Vec<Friend>;
}
