use crate::error::{PasswordManagerError, Result};
use std::{env, path::PathBuf};

/// Name of the encrypted file where passwords are stored.
const ENC_FILE: &str = ".pwd.enc";

/// Returns the full path to the encrypted storage file in the user's home directory.
pub fn get_file_path() -> Result<PathBuf> {
    let home_dir = env::var("HOME").map_err(|_| {
        PasswordManagerError::NotFound("HOME environment variable not set".to_string())
    })?;
    Ok(PathBuf::from(home_dir).join(ENC_FILE))
}
