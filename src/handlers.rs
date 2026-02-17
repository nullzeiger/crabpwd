use crate::app;
use crate::cli::{AddArgs, DeleteArgs, ListArgs, SearchArgs};
use crate::error::{PasswordManagerError, Result};
use crate::ui;
use std::io::{self, Write};

/// Prompts the user to enter a value interactively.
fn prompt(label: &str) -> Result<String> {
    print!("{}: ", label);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Handles the `list` subcommand.
pub fn handle_list(args: ListArgs) -> Result<()> {
    let results = app::list_passwords(args.limit)?;
    ui::print_password_list(&results);
    Ok(())
}

/// Handles the `add` subcommand, with optional interactive mode.
pub fn handle_add(mut args: AddArgs) -> Result<()> {
    if args.interactive || args.are_any_fields_missing() {
        if args.website.is_none() {
            args.website = Some(prompt("Website")?);
        }
        if args.username.is_none() {
            args.username = Some(prompt("Username")?);
        }
        if args.email.is_none() {
            args.email = Some(prompt("Email")?);
        }
        if args.password.is_none() {
            args.password =
                Some(rpassword::prompt_password("Password: ").map_err(PasswordManagerError::Io)?);
        }
    }

    let password = args.to_password()?;
    app::add_password(password)?;
    println!("Password added successfully.");
    Ok(())
}

/// Handles the `delete` subcommand.
pub fn handle_delete(args: DeleteArgs) -> Result<()> {
    app::delete_password(args.index)?;
    println!("Password at index {} deleted.", args.index);
    Ok(())
}

/// Handles the `search` subcommand.
pub fn handle_search(args: SearchArgs) -> Result<()> {
    let query = args.query.ok_or_else(|| {
        PasswordManagerError::InvalidFormat("Search query is required".to_string())
    })?;
    let results = app::search_passwords(&query)?;
    ui::print_password_list(&results);
    Ok(())
}
