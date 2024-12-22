use std::{collections::HashMap, error::Error};
use std::time::Instant;
use horse_protocol::{Friend, FriendStatus, UserId};

use crate::{database::Database, utils::{is_valid_email, validate_account_name, validate_display_name, validate_password}};

#[derive(Clone, Debug)]
pub struct User {
    pub id: UserId,
    pub created_at: u32,
    pub account_name: String,
    pub account_name_lower: String,
    pub display_name: String,
    pub email: String,
    pub password: String,
}

pub struct DataService<T> 
    where T: Database
{
    database: T,
}

impl<T> DataService<T> 
    where T: Database 
{
    pub fn new(database: T) -> Self {
        Self {
            database,
        }
    }

    pub fn create_user(
        &mut self, 
        user_id: UserId, 
        account_name: String, 
        email: String, 
        display_name: String,
        password: String
    ) -> Result<(), Box<dyn Error>> {

        validate_password(&password)?;
        validate_account_name(&account_name)?;
        validate_display_name(&display_name)?;

        if !is_valid_email(&email) {
            return Err("Invalid email".into());
        }

        if self.database.user_get_by_account_name(&account_name).is_some() {
            return Err("Account name already exists".into());
        }

        if self.database.user_get_by_email(&email).is_some() {
            return Err("Email already exists".into());
        }

        let account_name_lower = account_name.to_lowercase();
        self.database.user_insert(User {
            id: user_id.clone(),
            created_at: 0,
            account_name,
            account_name_lower,
            email,
            display_name,
            password,
        });

        Ok(())
    }

    pub fn get_friends_list(&self, user_id: UserId) -> Vec<Friend> {
        self.database.friends_get(user_id)
    }

    pub fn add_friend(&mut self, user_id: UserId, friend_id: UserId) {
        self.database.friends_add(user_id, friend_id);
    }

    pub fn remove_friend(&mut self, user_id: UserId, friend_id: UserId) {
        self.database.friends_remove(user_id, friend_id);
    }

    pub fn get_user(&self, user_id: UserId) -> Option<User> {
        self.database.user_get(user_id)
    }
}
