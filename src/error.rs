use std::fmt;

/// A custom error type for all possible failures in the application.
#[derive(Debug)]
pub enum PasswordManagerError {
    Io(std::io::Error),
    Parse(String),
    NotFound(String),
    InvalidFormat(String),
}

/// Formats the error for a user-friendly display in the terminal.
impl fmt::Display for PasswordManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PasswordManagerError::Io(err) => write!(f, "IO error: {}", err),
            PasswordManagerError::Parse(msg) => write!(f, "Parse error: {}", msg),
            PasswordManagerError::NotFound(msg) => write!(f, "Not found: {}", msg),
            PasswordManagerError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

/// Allows for the automatic conversion from `std::io::Error` into our custom error type.
impl From<std::io::Error> for PasswordManagerError {
    fn from(err: std::io::Error) -> Self {
        PasswordManagerError::Io(err)
    }
}

/// A convenient type alias for `std::result::Result` using our custom error type.
pub type Result<T> = std::result::Result<T, PasswordManagerError>;
