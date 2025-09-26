use crate::app;
use crate::cli::{AddArgs, DeleteArgs, ListArgs, SearchArgs};
use crate::error::{PasswordManagerError, Result};
use crate::models::Password;
use crate::ui;
use std::process;

/// Handles the 'list' subcommand.
pub fn handle_list(args: ListArgs) -> Result<()> {
    let passwords = app::list_passwords(args.limit)?;
    if passwords.is_empty() {
        println!("No passwords stored yet. Use 'crabpwd add' to add one.");
        return Ok(());
    }
    match args.limit {
        Some(limit) => println!("Showing first {} password(s):", limit),
        None => println!("All stored passwords:"),
    }
    println!();
    ui::print_passwords(&passwords);
    Ok(())
}

/// Handles the 'add' subcommand.
pub fn handle_add(args: AddArgs) -> Result<()> {
    let password = if args.interactive || args.are_any_fields_missing() {
        collect_password_interactively(args)?
    } else {
        args.to_password()?
    };
    app::add_password(password)?;
    println!("✓ Password added successfully!");
    Ok(())
}

/// Handles the 'delete' subcommand.
pub fn handle_delete(args: DeleteArgs) -> Result<()> {
    let passwords = app::list_passwords(None)?;
    let password_to_delete = passwords.iter().find(|(idx, _)| *idx == args.index);

    if let Some((_, password)) = password_to_delete {
        println!("You are about to delete:");
        println!("  Website: {}", password.website);
        println!("  Username: {}", password.username);
        if ui::confirm_action("Are you sure you want to delete this password?")? {
            app::delete_password(args.index)?;
            println!("✓ Password deleted successfully!");
        } else {
            println!("Deletion cancelled.");
        }
    } else {
        return Err(PasswordManagerError::NotFound(format!(
            "No password found at index {}.",
            args.index
        )));
    }
    Ok(())
}

/// Handles the 'search' subcommand.
pub fn handle_search(args: SearchArgs) -> Result<()> {
    let query = match args.query {
        Some(q) if !q.trim().is_empty() => q,
        _ => return Err(PasswordManagerError::InvalidFormat(
            "Please provide a search query.".to_string(),
        )),
    };
    let results = app::search_passwords(&query)?;
    if results.is_empty() {
        println!("No passwords found matching '{}'", query);
    } else {
        println!("Found {} password(s) matching '{}':", results.len(), query);
        println!();
        ui::print_passwords(&results);
    }
    Ok(())
}

/// Collects password data interactively.
fn collect_password_interactively(args: AddArgs) -> Result<Password> {
    println!("=== Adding New Password (Interactive) ===");
    let website = args.website.unwrap_or_else(|| {
        ui::prompt_for_input("Website/Service", true).unwrap_or_else(|_| process::exit(1))
    });
    let username = args.username.unwrap_or_else(|| {
        ui::prompt_for_input("Username", true).unwrap_or_else(|_| process::exit(1))
    });
    let email = args.email.unwrap_or_else(|| {
        ui::prompt_for_input("Email", true).unwrap_or_else(|_| process::exit(1))
    });
    let password = args.password.unwrap_or_else(|| {
        ui::prompt_for_password("Password").unwrap_or_else(|_| process::exit(1))
    });
    println!();
    Password::new(website, username, email, password)
}
