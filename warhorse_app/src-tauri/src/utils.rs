pub enum UserLoginType {
    AccountName,
    Email,
}

pub fn is_username_or_email(username: &str) -> UserLoginType {
    if username.contains('@') {
        UserLoginType::Email
    } else {
        UserLoginType::AccountName
    }
}