use crate::error::{PasswordManagerError, Result};
use serde::{Deserialize, Serialize};

/// Represents a single password entry in the store.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Password {
    pub website: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

impl Password {
    /// Creates a new `Password`, validating that no field is empty.
    pub fn new(website: String, username: String, email: String, password: String) -> Result<Self> {
        if website.is_empty() {
            return Err(PasswordManagerError::InvalidFormat(
                "Website cannot be empty".to_string(),
            ));
        }
        if username.is_empty() {
            return Err(PasswordManagerError::InvalidFormat(
                "Username cannot be empty".to_string(),
            ));
        }
        if email.is_empty() {
            return Err(PasswordManagerError::InvalidFormat(
                "Email cannot be empty".to_string(),
            ));
        }
        if password.is_empty() {
            return Err(PasswordManagerError::InvalidFormat(
                "Password cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            website,
            username,
            email,
            password,
        })
    }
}
