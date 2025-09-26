use crate::error::{PasswordManagerError, Result};
use crate::models::Password;
use std::env;
use std::fs::File;
use std::path::PathBuf;

/// The name of the JSON file where passwords will be stored.
const JSON_FILE: &str = ".pwd.json";

/// Constructs the full path to the JSON storage file.
fn get_json_path() -> Result<PathBuf> {
    let home_dir = env::var("HOME").map_err(|_| {
        PasswordManagerError::NotFound("HOME environment variable not set".to_string())
    })?;
    Ok(PathBuf::from(home_dir).join(JSON_FILE))
}

/// Reads and deserializes passwords from the JSON file.
pub fn load_passwords() -> Result<Vec<Password>> {
    let path = get_json_path()?;
    if !path.exists() {
        // The file doesn't exist, so return an empty vector.
        return Ok(Vec::new());
    }
    let file = File::open(path)?;
    serde_json::from_reader(file).map_err(|e| PasswordManagerError::Parse(e.to_string()))
}

/// Serializes the password list and writes it to the JSON file.
pub fn save_passwords(passwords: &[Password]) -> Result<()> {
    let path = get_json_path()?;
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, passwords)
        .map_err(|e| PasswordManagerError::Parse(e.to_string()))
}
