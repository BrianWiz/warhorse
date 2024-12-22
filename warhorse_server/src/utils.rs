use regex::Regex;
use warhorse_protocol::Language;
use warhorse_protocol::{ACCOUNT_NAME_MAX_LENGTH, ACCOUNT_NAME_MIN_LENGTH, DISPLAY_NAME_MAX_LENGTH, DISPLAY_NAME_MIN_LENGTH, PASSWORD_MIN_LENGTH};
use crate::error::ServerError;

pub fn validate_password(password: &String, language: Language) -> Result<(), ServerError> {
    if password.len() < PASSWORD_MIN_LENGTH {
        return Err(crate::i18n::invalid_password(language));
    }
    Ok(())
}

pub fn validate_account_name(account_name: &String, language: Language) -> Result<(), ServerError> {
    if account_name.len() < ACCOUNT_NAME_MIN_LENGTH || account_name.len() > ACCOUNT_NAME_MAX_LENGTH {
        return Err(crate::i18n::invalid_account_name(language));
    }
    Ok(())
}

pub fn validate_display_name(display_name: &String, language: Language) -> Result<(), ServerError> {
    if display_name.len() < DISPLAY_NAME_MIN_LENGTH  || display_name.len() > DISPLAY_NAME_MAX_LENGTH {
        return Err(crate::i18n::invalid_display_name(language));
    }
    Ok(())
}

pub fn is_valid_email(email: &String) -> bool {

    // thanks AI!
    let email_regex = Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();

    if email.len() > 254 {
        return false;
    }

    email_regex.is_match(email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(is_valid_email(&"test@example.com".to_string()));
        assert!(is_valid_email(&"user.name+tag@example.com".to_string()));
        assert!(is_valid_email(&"user_name@example.co.uk".to_string()));
        assert!(is_valid_email(&"test.email-with-dash@example.com".to_string()));
        assert!(is_valid_email(&"a@b.com".to_string()));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!is_valid_email(&"@example.com".to_string()));
        assert!(!is_valid_email(&"test@".to_string()));
        assert!(!is_valid_email(&"test".to_string()));
        assert!(!is_valid_email(&"test@.com".to_string()));
        assert!(!is_valid_email(&"test@com.".to_string()));
        assert!(!is_valid_email(&"test space@example.com".to_string()));
        assert!(!is_valid_email(&"".to_string()));
        assert!(!is_valid_email(&"test@example..com".to_string()));

        // Test email length > 254 characters
        let long_email = format!("{}@example.com", "a".repeat(250));
        assert!(!is_valid_email(&long_email));
    }
}