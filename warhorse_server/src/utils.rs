use regex::Regex;

pub fn validate_password(password: &String) -> Result<(), Box<dyn std::error::Error>> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".into());
    }
    Ok(())
}

pub fn validate_account_name(username: &String) -> Result<(), Box<dyn std::error::Error>> {
    if username.len() < 3  || username.len() > 20 {
        return Err("Username must be between 3 and 20 characters".into());
    }
    Ok(())
}

pub fn validate_display_name(display_name: &String) -> Result<(), Box<dyn std::error::Error>> {
    if display_name.len() < 3  || display_name.len() > 20 {
        return Err("Display name must be between 3 and 20 characters".into());
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