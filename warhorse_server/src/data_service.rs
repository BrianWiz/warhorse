use warhorse_protocol::{Friend, UserId, UserRegistration, UserPartial};
use crate::error::ServerError;
use crate::{database::Database, utils::{is_valid_email, validate_account_name, validate_display_name, validate_password}};

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
        req: UserRegistration,
    ) -> Result<UserId, ServerError> {

        validate_password(&req.password, req.language)?;
        validate_account_name(&req.account_name, req.language)?;
        validate_display_name(&req.display_name, req.language)?;

        if !is_valid_email(&req.email) {
            return Err(crate::i18n::invalid_email(req.language));
        }

        if self.database.users_get_by_account_name(&req.account_name).is_some() {
            return Err(crate::i18n::account_name_already_exists(req.language));
        }

        if self.database.users_get_by_email(&req.email).is_some() {
            return Err(crate::i18n::email_already_exists(req.language));
        }

        let new_user_id = self.database.users_insert(req);
        Ok(new_user_id)
    }

    pub fn friends_get(&self, user_id: UserId) -> Vec<Friend> {
        self.database.friends_get(user_id)
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
        self.database.user_blocks_insert(user_id, blocked_id);
    }
    
    pub fn user_blocks_remove(&mut self, user_id: UserId, blocked_id: UserId) {
        self.database.user_blocks_remove(user_id, blocked_id);
    }
}
