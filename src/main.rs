// --- Crates and Modules ---
// Import necessary external crates and standard library modules.
use clap::{Args, Parser, Subcommand}; // For parsing command-line arguments.
use serde::{Deserialize, Serialize}; // For serializing and deserializing data (to/from JSON).
use std::env; // For accessing environment variables (like the HOME directory).
use std::fmt; // For custom formatting (e.g., for the Error type).
use std::fs::File; // For file I/O operations.
use std::io::{self, Write}; // For standard input/output, including flushing stdout.
use std::path::PathBuf; // For handling file system paths in a cross-platform way.
use std::process; // For exiting the process with a specific status code.

// --- Constants ---
/// The name of the JSON file where passwords will be stored.
const JSON_FILE: &str = ".pwd.json";

// --- Command-Line Interface Definition ---

/// Defines the main command-line interface structure using clap.
/// This is the top-level command that holds all subcommands.
#[derive(Parser)]
#[command(
    name = "crabpwd",
    version = "0.1.0",
    author = "Ivan Guerreschi",
    about = "Password manager written in Rust",
    long_about = "crabpwd is a command-line password manager."
)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Enumerates the available subcommands for the application.
#[derive(Subcommand)]
enum Commands {
    /// List all passwords (or first N passwords)
    List(ListArgs),
    /// Add a new password entry
    Add(AddArgs),
    /// Delete a password by index
    Delete(DeleteArgs),
    /// Search passwords by website, username, or email
    Search(SearchArgs),
}

/// Defines the arguments for the 'list' subcommand.
#[derive(Args)]
struct ListArgs {
    /// Number of passwords to display (default: all)
    #[arg(short, long)]
    limit: Option<usize>,
}

/// Defines the arguments for the 'delete' subcommand.
#[derive(Args)]
struct DeleteArgs {
    /// Index of the password to delete (1-based)
    #[arg(value_name = "INDEX")]
    index: usize,
}

/// Defines the arguments for the 'add' subcommand.
#[derive(Args)]
struct AddArgs {
    /// Website or service name
    #[arg(short, long)]
    website: Option<String>,
    /// Username for the account
    #[arg(short = 'u', long)]
    username: Option<String>,
    /// Email address for the account
    #[arg(short, long)]
    email: Option<String>,
    /// Password for the account
    #[arg(short, long)]
    password: Option<String>,
    /// Interactive mode - prompt for missing fields
    #[arg(short, long)]
    interactive: bool,
}

/// Defines the arguments for the 'search' subcommand.
#[derive(Args)]
struct SearchArgs {
    /// Search term to match against website, username, or email
    #[arg(value_name = "QUERY")]
    query: Option<String>,
}

// --- Custom Error and Result Types ---

/// A custom error type for all possible failures in the application.
/// This consolidates different error kinds into a single, manageable type.
#[derive(Debug)]
pub enum PasswordManagerError {
    Io(std::io::Error),
    Parse(String),
    NotFound(String),
    InvalidFormat(String),
}

/// Formats the error for user-friendly display in the terminal.
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
/// This is a convenient way to handle I/O errors using the `?` operator.
impl From<std::io::Error> for PasswordManagerError {
    fn from(err: std::io::Error) -> Self {
        PasswordManagerError::Io(err)
    }
}

/// A convenient type alias for `std::result::Result` using our custom error type.
type Result<T> = std::result::Result<T, PasswordManagerError>;

// --- Data Structure ---

/// Represents a single password entry with its associated data.
/// It can be serialized to and deserialized from JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password {
    pub website: String,
    pub username: String,
    pub email: String,
    pub pwd: String,
}

impl Password {
    /// Creates a new `Password` instance after validating that no required fields are empty.
    /// Returns an `Err` if any field is just whitespace.
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

/// Defines how a `Password` entry is displayed as a string.
impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Website: {}, Username: {}, Email: {}, Password: {}",
            self.website, self.username, self.email, self.pwd
        )
    }
}

// --- File System Utilities ---

/// Resolves the path to the user's home directory.
fn get_home_dir() -> Result<PathBuf> {
    env::var("HOME").map(PathBuf::from).map_err(|_| {
        PasswordManagerError::NotFound("HOME environment variable not set".to_string())
    })
}

/// Constructs the full path to the JSON storage file (e.g., "/home/user/.pwd.json").
fn get_json_path() -> Result<PathBuf> {
    Ok(get_home_dir()?.join(JSON_FILE))
}

// --- Data Persistence Logic ---

/// Reads and deserializes passwords from the JSON file.
/// Returns an empty vector if the file doesn't exist yet.
fn load_passwords() -> Result<Vec<Password>> {
    let path = get_json_path()?;
    if !path.exists() {
        return Ok(Vec::new()); // No file, so no passwords. This is not an error.
    }
    let file = File::open(path)?;
    let passwords: Vec<Password> =
        serde_json::from_reader(file).map_err(|e| PasswordManagerError::Parse(e.to_string()))?;
    Ok(passwords)
}

/// Serializes the given password list and writes it to the JSON file.
/// This will overwrite any existing content in the file.
fn save_passwords(passwords: &[Password]) -> Result<()> {
    let path = get_json_path()?;
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, passwords) // `to_writer_pretty` for human-readable JSON.
        .map_err(|e| PasswordManagerError::Parse(e.to_string()))?;
    Ok(())
}

// --- Core Application Logic ---

/// Adds a new password to the store by loading, appending, and re-saving the list.
fn add_password(password: Password) -> Result<()> {
    let mut passwords = load_passwords()?;
    passwords.push(password);
    save_passwords(&passwords)?;
    println!("Password added successfully");
    Ok(())
}

/// Deletes a password from the store at a given 1-based index.
fn delete_password(index: usize) -> Result<()> {
    if index == 0 {
        return Err(PasswordManagerError::InvalidFormat(
            "Index must be greater than 0".to_string(),
        ));
    }

    let mut passwords = load_passwords()?;
    if index > passwords.len() {
        return Err(PasswordManagerError::NotFound(format!(
            "No password found at index {}",
            index
        )));
    }

    passwords.remove(index - 1); // Convert 1-based index to 0-based for vector access.
    save_passwords(&passwords)?;
    println!("Password at index {} deleted successfully", index);
    Ok(())
}

/// Searches for passwords matching a query string.
/// The search is case-insensitive and checks the website, username, and email fields.
fn search_passwords(query: &str) -> Result<Vec<(usize, Password)>> {
    let passwords = load_passwords()?;
    let query_lower = query.to_lowercase();
    let results = passwords
        .into_iter()
        .enumerate()
        .filter_map(|(i, pwd)| {
            if pwd.website.to_lowercase().contains(&query_lower)
                || pwd.username.to_lowercase().contains(&query_lower)
                || pwd.email.to_lowercase().contains(&query_lower)
            {
                Some((i + 1, pwd)) // Return with 1-based index for display.
            } else {
                None
            }
        })
        .collect();
    Ok(results)
}

/// Retrieves all stored passwords, optionally limited to a certain number.
fn list_passwords(limit: Option<usize>) -> Result<Vec<(usize, Password)>> {
    let passwords = load_passwords()?;
    let mut results: Vec<(usize, Password)> = passwords
        .into_iter()
        .enumerate()
        .map(|(i, pwd)| (i + 1, pwd)) // Map to 1-based index.
        .collect();

    if let Some(lim) = limit {
        results.truncate(lim); // Apply the limit if provided.
    }

    Ok(results)
}

/// A helper function to print a list of passwords to the console.
fn print_passwords(passwords: &[(usize, Password)]) {
    if passwords.is_empty() {
        println!("No passwords found.");
        return;
    }

    for (index, password) in passwords {
        println!("Index: {} - {}", index, password);
    }
}

// --- Main Application Flow ---

/// The standard main entry point for the program.
/// It calls the `run` function and handles any top-level errors gracefully.
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// The application's primary logic function.
/// It parses CLI arguments and dispatches to the appropriate handler function.
fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List(args) => handle_list(args),
        Commands::Add(args) => handle_add(args),
        Commands::Delete(args) => handle_delete(args),
        Commands::Search(args) => handle_search(args),
    }
}

// --- Subcommand Handlers ---

/// Handles the 'list' subcommand.
fn handle_list(args: ListArgs) -> Result<()> {
    let passwords = list_passwords(args.limit)?;

    if passwords.is_empty() {
        println!("No passwords stored yet. Use 'crabpwd add' to add your first password.");
        return Ok(());
    }

    match args.limit {
        Some(limit) => println!("Showing first {} password(s):", limit),
        None => println!("All stored passwords:"),
    }
    println!();

    print_passwords(&passwords);
    println!("\nTotal: {} password(s)", passwords.len());

    Ok(())
}

/// Handles the 'add' subcommand, deciding between interactive and non-interactive mode.
fn handle_add(args: AddArgs) -> Result<()> {
    let password = if args.interactive || are_any_fields_missing(&args) {
        // If interactive flag is set or any field is missing, start interactive mode.
        collect_password_interactively(args)?
    } else {
        // Otherwise, create the password directly from arguments.
        create_password_from_args(args)?
    };

    add_password(password)?;
    println!("✓ Password added successfully!");

    Ok(())
}

/// Handles the 'delete' subcommand, including a confirmation prompt.
fn handle_delete(args: DeleteArgs) -> Result<()> {
    if args.index == 0 {
        return Err(PasswordManagerError::InvalidFormat(
            "Index must be greater than 0. Use 'crabpwd list' to see available indices."
                .to_string(),
        ));
    }

    // Fetch the password to be deleted to show it in the confirmation prompt.
    let passwords = list_passwords(None)?;
    let password_to_delete = passwords
        .iter()
        .find(|(idx, _)| *idx == args.index)
        .map(|(_, pwd)| pwd);

    if let Some(password) = password_to_delete {
        println!("You are about to delete:");
        println!("  Website: {}", password.website);
        println!("  Username: {}", password.username);

        if confirm_action("Are you sure you want to delete this password?")? {
            delete_password(args.index)?;
            println!("✓ Password deleted successfully!");
        } else {
            println!("Deletion cancelled.");
        }
    } else {
        return Err(PasswordManagerError::NotFound(format!(
            "No password found at index {}. Use 'crabpwd list' to see available indices.",
            args.index
        )));
    }

    Ok(())
}

/// Handles the 'search' subcommand.
fn handle_search(args: SearchArgs) -> Result<()> {
    let query = match args.query {
        Some(q) if !q.trim().is_empty() => q,
        _ => {
            return Err(PasswordManagerError::InvalidFormat(
                "Please provide a search query. Example: 'crabpwd search github'".to_string(),
            ));
        }
    };

    let results = search_passwords(&query)?;

    if results.is_empty() {
        println!("No passwords found matching '{}'", query);
        println!("Try searching with a different term or use 'crabpwd list' to see all passwords.");
    } else {
        println!("Found {} password(s) matching '{}':", results.len(), query);
        println!();
        print_passwords(&results);
    }

    Ok(())
}

// --- Interactive Mode Helpers ---

/// Checks if any of the required fields are missing from the command-line arguments.
fn are_any_fields_missing(args: &AddArgs) -> bool {
    args.website.is_none()
        || args.username.is_none()
        || args.email.is_none()
        || args.password.is_none()
}

/// Constructs a `Password` struct from the provided command-line arguments.
/// Returns an error if any required argument is missing.
fn create_password_from_args(args: AddArgs) -> Result<Password> {
    let website = args
        .website
        .ok_or_else(|| PasswordManagerError::InvalidFormat("Website is required".to_string()))?;

    let username = args
        .username
        .ok_or_else(|| PasswordManagerError::InvalidFormat("Username is required".to_string()))?;

    let email = args
        .email
        .ok_or_else(|| PasswordManagerError::InvalidFormat("Email is required".to_string()))?;

    let password = args
        .password
        .ok_or_else(|| PasswordManagerError::InvalidFormat("Password is required".to_string()))?;

    Password::new(website, username, email, password)
}

/// Prompts the user to enter each password field interactively.
/// It uses values from arguments if they were provided, otherwise it prompts for them.
fn collect_password_interactively(args: AddArgs) -> Result<Password> {
    println!("=== Adding New Password ===");
    println!("Fill in the required information:");
    println!();

    let website = args.website.unwrap_or_else(|| {
        prompt_for_input("Website/Service (required)", true).unwrap_or_else(|_| process::exit(1))
    });

    let username = args.username.unwrap_or_else(|| {
        prompt_for_input("Username (required)", true).unwrap_or_else(|_| process::exit(1))
    });

    let email = args.email.unwrap_or_else(|| {
        prompt_for_input("Email (required)", false).unwrap_or_else(|_| process::exit(1))
    });

    // Use the secure password prompt.
    let password = args.password.unwrap_or_else(|| {
        prompt_for_password("Password (required)").unwrap_or_else(|_| process::exit(1))
    });

    println!();
    Password::new(website, username, email, password)
}

/// Prompts the user for a password securely without echoing input to the terminal.
fn prompt_for_password(prompt: &str) -> io::Result<String> {
    loop {
        // `rpassword::prompt_password` handles hiding the input.
        let input = rpassword::prompt_password(format!("{}: ", prompt))?;
        if input.trim().is_empty() {
            println!("This field is required. Please enter a value.");
            continue; // Re-prompt if the input is empty.
        }
        return Ok(input.trim().to_string());
    }
}

/// Prompts the user for standard input, echoing it to the terminal.
/// Includes validation to ensure required fields are not empty.
fn prompt_for_input(prompt: &str, required: bool) -> io::Result<String> {
    loop {
        print!("{}: ", prompt);
        io::stdout().flush()?; // Ensure the prompt is displayed before waiting for input.

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input.is_empty() && required {
            println!("This field is required. Please enter a value.");
            continue; // Re-prompt.
        }

        return Ok(input);
    }
}

/// Displays a confirmation prompt and waits for a yes/no answer from the user.
fn confirm_action(message: &str) -> io::Result<bool> {
    loop {
        print!("{} (y/N): ", message);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" | "" => return Ok(false), // Default to 'no' if Enter is pressed.
            _ => println!("Please enter 'y' for yes or 'n' for no."),
        }
    }
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::try_parse_from(&["crabpwd", "list", "--limit", "5"]);
        assert!(cli.is_ok());

        let cli = Cli::try_parse_from(&["crabpwd", "add", "--website", "github.com"]);
        assert!(cli.is_ok());

        let cli = Cli::try_parse_from(&["crabpwd", "delete", "1"]);
        assert!(cli.is_ok());

        let cli = Cli::try_parse_from(&["crabpwd", "search", "github"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_missing_fields_detection() {
        let args = AddArgs {
            website: Some("test.com".to_string()),
            username: None,
            email: None,
            password: None,
            interactive: false,
        };
        assert!(are_any_fields_missing(&args));

        let args = AddArgs {
            website: Some("test.com".to_string()),
            username: Some("user".to_string()),
            email: Some("email".to_string()),
            password: Some("pass".to_string()),
            interactive: false,
        };
        assert!(!are_any_fields_missing(&args));
    }

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
