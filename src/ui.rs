use crate::models::Password;
use std::io::{self, Write};

/// Prints a list of passwords to the console.
pub fn print_passwords(passwords: &[(usize, Password)]) {
    if passwords.is_empty() {
        println!("No passwords found.");
        return;
    }
    for (index, password) in passwords {
        println!("Index: {} - {}", index, password);
    }
}

/// Prompts the user for a password securely (without echoing).
pub fn prompt_for_password(prompt: &str) -> io::Result<String> {
    loop {
        let input = rpassword::prompt_password(format!("{}: ", prompt))?;
        if input.trim().is_empty() {
            println!("This field is required. Please enter a value.");
            continue;
        }
        return Ok(input.trim().to_string());
    }
}

/// Prompts the user for standard input.
pub fn prompt_for_input(prompt: &str, required: bool) -> io::Result<String> {
    loop {
        print!("{}: ", prompt);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();
        if input.is_empty() && required {
            println!("This field is required. Please enter a value.");
            continue;
        }
        return Ok(input);
    }
}

/// Displays a confirmation prompt and waits for a yes/no answer.
pub fn confirm_action(message: &str) -> io::Result<bool> {
    loop {
        print!("{} (y/N): ", message);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" | "" => return Ok(false), // Default to 'no'
            _ => println!("Please enter 'y' for yes or 'n' for no."),
        }
    }
}
