use warhorse_protocol::{Friend, UserPartial, UserId, UserRegistration};

use super::Database;

pub struct PostgresDatabase {}

impl Database for PostgresDatabase {

    fn new(_connection_string: &str) -> Self {
        unimplemented!();
    }

    fn user_exists(&self, user_id: UserId) -> bool {
        unimplemented!();
    }

    fn users_insert(&mut self, user: UserRegistration) -> UserId {
        unimplemented!();
    }

    fn users_get(&self, user_id: UserId) -> Option<UserPartial> {
        unimplemented!();
    }

    fn users_get_by_account_name(&self, account_name: &str) -> Option<UserPartial> {
        unimplemented!();
    }

    fn users_get_by_email(&self, email: &str) -> Option<UserPartial> {
        unimplemented!();
    }

    fn user_blocks_insert(&mut self, user_id: UserId, blocked_id: UserId) {
        unimplemented!();
    }

    fn user_blocks_remove(&mut self, user_id: UserId, blocked_id: UserId) {
        unimplemented!();
    }

    fn user_blocks_get_blocks_for_user(&self, user_id: UserId) -> Vec<UserPartial> {
        unimplemented!();
    }

    fn user_is_blocked(&self, user_id: UserId, blocked_id: UserId) -> bool {
        unimplemented!();
    }

    fn friend_requests_insert(&mut self, user_id: UserId, friend_id: UserId) {
        unimplemented!();
    }

    fn friend_requests_remove(&mut self, user_id: UserId, friend_id: UserId) {
        unimplemented!();
    }

    fn friend_requests_get(&self, user_id: UserId) -> Vec<Friend> {
        unimplemented!();
    }

    fn friends_add(&mut self, user_id: UserId, friend_id: UserId) {
        unimplemented!();
    }

    fn friends_remove(&mut self, user_id: UserId, friend_id: UserId) {
        unimplemented!();
    }

    fn friends_get(&self, user_id: UserId) -> Vec<Friend> {
        unimplemented!();
    }
}

