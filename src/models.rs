use crate::error::{PasswordManagerError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a single password entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password {
    pub website: String,
    pub username: String,
    pub email: String,
    pub pwd: String,
}

impl Password {
    /// Creates a new `Password` instance after validating the fields.
    pub fn new(website: String, username: String, email: String, pwd: String) -> Result<Self> {
        if website.trim().is_empty() {
            return Err(PasswordManagerError::InvalidFormat(
                "Website cannot be empty".to_string(),
            ));
        }
        if username.trim().is_empty() {
            return Err(PasswordManagerError::InvalidFormat(
                "Username cannot be empty".to_string(),
            ));
        }
        if email.trim().is_empty() {
            return Err(PasswordManagerError::InvalidFormat(
                "Email cannot be empty".to_string(),
            ));
        }
        if pwd.trim().is_empty() {
            return Err(PasswordManagerError::InvalidFormat(
                "Password cannot be empty".to_string(),
            ));
        }

        Ok(Password {
            website: website.trim().to_string(),
            username: username.trim().to_string(),
            email: email.trim().to_string(),
            pwd: pwd.trim().to_string(),
        })
    }
}

/// Defines how a `Password` is displayed as a string.
impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Website: {}, Username: {}, Email: {}, Password: {}",
            self.website, self.username, self.email, self.pwd
        )
    }
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_creation() {
        let password = Password::new(
            "github.com".to_string(),
            "user123".to_string(),
            "user@example.com".to_string(),
            "secret123".to_string(),
        );
        assert!(password.is_ok());
    }

    #[test]
    fn test_empty_website_validation() {
        let password = Password::new(
            "  ".to_string(), // Test with whitespace
            "user123".to_string(),
            "user@example.com".to_string(),
            "secret123".to_string(),
        );
        assert!(password.is_err());
    }

    #[test]
    fn test_json_serialization() {
        let password = Password::new(
            "github.com".to_string(),
            "user123".to_string(),
            "user@example.com".to_string(),
            "secret123".to_string(),
        )
        .unwrap();

        let json = serde_json::to_string(&password).unwrap();
        let decoded: Password = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.website, "github.com");
        assert_eq!(decoded.username, "user123");
    }
}
