use clap::{Args, Parser, Subcommand};
use crate::error::{PasswordManagerError, Result};
use crate::models::Password;

/// Defines the main command-line interface structure using clap.
#[derive(Parser)]
#[command(
    name = "crabpwd",
    version = "0.1.0",
    author = "Ivan Guerreschi",
    about = "Password manager written in Rust",
    long_about = "crabpwd is a command-line password manager."
)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Enumerates the available subcommands for the application.
#[derive(Subcommand)]
pub enum Commands {
    /// List all passwords (or first N passwords)
    List(ListArgs),
    /// Add a new password entry
    Add(AddArgs),
    /// Delete a password by index
    Delete(DeleteArgs),
    /// Search passwords by website, username, or email
    Search(SearchArgs),
}

#[derive(Args)]
pub struct ListArgs {
    /// Number of passwords to display (default: all)
    #[arg(short, long)]
    pub limit: Option<usize>,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Index of the password to delete (1-based)
    #[arg(value_name = "INDEX")]
    pub index: usize,
}

#[derive(Args)]
pub struct AddArgs {
    /// Website or service name
    #[arg(short, long)]
    pub website: Option<String>,
    /// Username for the account
    #[arg(short = 'u', long)]
    pub username: Option<String>,
    /// Email address for the account
    #[arg(short, long)]
    pub email: Option<String>,
    /// Password for the account
    #[arg(short, long)]
    pub password: Option<String>,
    /// Interactive mode - prompt for missing fields
    #[arg(short, long)]
    pub interactive: bool,
}

impl AddArgs {
    /// Checks if any of the required fields are missing.
    pub fn are_any_fields_missing(&self) -> bool {
        self.website.is_none()
            || self.username.is_none()
            || self.email.is_none()
            || self.password.is_none()
    }

    /// Tries to build a `Password` from the arguments.
    pub fn to_password(self) -> Result<Password> {
        let website = self
            .website
            .ok_or_else(|| PasswordManagerError::InvalidFormat("Website is required".to_string()))?;
        let username = self
            .username
            .ok_or_else(|| PasswordManagerError::InvalidFormat("Username is required".to_string()))?;
        let email = self
            .email
            .ok_or_else(|| PasswordManagerError::InvalidFormat("Email is required".to_string()))?;
        let password = self
            .password
            .ok_or_else(|| PasswordManagerError::InvalidFormat("Password is required".to_string()))?;

        Password::new(website, username, email, password)
    }
}

#[derive(Args)]
pub struct SearchArgs {
    /// Search term to match against website, username, or email
    #[arg(value_name = "QUERY")]
    pub query: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        assert!(Cli::try_parse_from(&["crabpwd", "list", "--limit", "5"]).is_ok());
        assert!(Cli::try_parse_from(&["crabpwd", "add", "-w", "github.com"]).is_ok());
        assert!(Cli::try_parse_from(&["crabpwd", "delete", "1"]).is_ok());
        assert!(Cli::try_parse_from(&["crabpwd", "search", "github"]).is_ok());
    }

    #[test]
    fn test_missing_fields_detection() {
        let args = AddArgs {
            website: Some("test.com".to_string()),
            username: None, email: None, password: None, interactive: false,
        };
        assert!(args.are_any_fields_missing());

        let args_full = AddArgs {
            website: Some("test.com".to_string()),
            username: Some("user".to_string()),
            email: Some("email".to_string()),
            password: Some("pass".to_string()),
            interactive: false,
        };
        assert!(!args_full.are_any_fields_missing());
    }
}
