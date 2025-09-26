use crate::error::{PasswordManagerError, Result};
use crate::models::Password;
use crate::storage;

/// Adds a new password to the store.
pub fn add_password(password: Password) -> Result<()> {
    let mut passwords = storage::load_passwords()?;
    passwords.push(password);
    storage::save_passwords(&passwords)
}

/// Deletes a password given its 1-based index.
pub fn delete_password(index: usize) -> Result<()> {
    if index == 0 {
        return Err(PasswordManagerError::InvalidFormat(
            "Index must be greater than 0".to_string(),
        ));
    }
    let mut passwords = storage::load_passwords()?;
    if index > passwords.len() {
        return Err(PasswordManagerError::NotFound(format!(
            "No password found at index {}",
            index
        )));
    }
    passwords.remove(index - 1);
    storage::save_passwords(&passwords)
}

/// Searches for passwords that match a query.
pub fn search_passwords(query: &str) -> Result<Vec<(usize, Password)>> {
    let passwords = storage::load_passwords()?;
    let query_lower = query.to_lowercase();
    let results = passwords
        .into_iter()
        .enumerate()
        .filter_map(|(i, pwd)| {
            if pwd.website.to_lowercase().contains(&query_lower)
                || pwd.username.to_lowercase().contains(&query_lower)
                || pwd.email.to_lowercase().contains(&query_lower)
            {
                Some((i + 1, pwd)) // 1-based index
            } else {
                None
            }
        })
        .collect();
    Ok(results)
}

/// Retrieves all passwords, with an optional limit.
pub fn list_passwords(limit: Option<usize>) -> Result<Vec<(usize, Password)>> {
    let passwords = storage::load_passwords()?;
    let mut results: Vec<(usize, Password)> = passwords
        .into_iter()
        .enumerate()
        .map(|(i, pwd)| (i + 1, pwd)) // 1-based index
        .collect();
    if let Some(lim) = limit {
        results.truncate(lim);
    }
    Ok(results)
}
