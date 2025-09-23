use clap::{Args, Parser, Subcommand};
use std::env;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process;

const CSV_FILE: &str = ".pwd.csv";
const CSV_TEMP_FILE: &str = ".pwd_tmp.csv";

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

#[derive(Args)]
struct ListArgs {
    /// Number of passwords to display (default: all)
    #[arg(short, long)]
    limit: Option<usize>,
}

#[derive(Args)]
struct DeleteArgs {
    /// Index of the password to delete (1-based)
    #[arg(value_name = "INDEX")]
    index: usize,
}

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

#[derive(Args)]
struct SearchArgs {
    /// Search term to match against website, username, or email
    #[arg(value_name = "QUERY")]
    query: Option<String>,
}

#[derive(Debug)]
pub enum PasswordManagerError {
    Io(std::io::Error),
    Parse(String),
    NotFound(String),
    InvalidFormat(String),
}

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

impl From<std::io::Error> for PasswordManagerError {
    fn from(err: std::io::Error) -> Self {
        PasswordManagerError::Io(err)
    }
}

type Result<T> = std::result::Result<T, PasswordManagerError>;

#[derive(Debug, Clone)]
pub struct Password {
    pub website: String,
    pub username: String,
    pub email: String,
    pub pwd: String,
}

impl Password {
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

    fn to_csv_line(&self) -> String {
        format!(
            "{},{},{},{}\n",
            self.website, self.username, self.email, self.pwd
        )
    }

    fn from_csv_line(line: &str) -> Result<Self> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(PasswordManagerError::InvalidFormat(format!(
                "Expected 4 fields, found {}",
                parts.len()
            )));
        }

        Password::new(
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
            parts[3].to_string(),
        )
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Website: {}, Username: {}, Email: {}, Password: {}",
            self.website, self.username, self.email, self.pwd
        )
    }
}

fn get_home_dir() -> Result<PathBuf> {
    env::var("HOME").map(PathBuf::from).map_err(|_| {
        PasswordManagerError::NotFound("HOME environment variable not set".to_string())
    })
}

fn get_csv_path() -> Result<PathBuf> {
    Ok(get_home_dir()?.join(CSV_FILE))
}

fn get_temp_csv_path() -> Result<PathBuf> {
    Ok(get_home_dir()?.join(CSV_TEMP_FILE))
}

fn create_if_not_exists() -> Result<()> {
    let path = get_csv_path()?;
    if !path.exists() {
        File::create(path)?;
    }
    Ok(())
}

fn open_for_reading() -> Result<File> {
    let path = get_csv_path()?;
    File::open(path).map_err(PasswordManagerError::from)
}

fn open_for_appending() -> Result<File> {
    let path = get_csv_path()?;
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(PasswordManagerError::from)
}

fn add_password(password: Password) -> Result<()> {
    let mut file = open_for_appending()?;
    file.write_all(password.to_csv_line().as_bytes())?;
    println!("Password added successfully");
    Ok(())
}

fn delete_password(index: usize) -> Result<()> {
    if index == 0 {
        return Err(PasswordManagerError::InvalidFormat(
            "Index must be greater than 0".to_string(),
        ));
    }

    let file = open_for_reading()?;
    let reader = BufReader::new(file);
    let mut passwords = Vec::new();

    // Read all passwords except the one to delete
    for (i, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        if !line.trim().is_empty() && i + 1 != index {
            passwords.push(line);
        }
    }

    // Write to temporary file
    let temp_path = get_temp_csv_path()?;
    fs::write(&temp_path, passwords.join("\n") + "\n")?;

    // Replace original file with temporary file
    let original_path = get_csv_path()?;
    fs::rename(temp_path, original_path)?;

    println!("Password at index {} deleted successfully", index);
    Ok(())
}

fn search_passwords(query: &str) -> Result<Vec<(usize, Password)>> {
    let file = open_for_reading()?;
    let reader = BufReader::new(file);
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    for (index, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }

        match Password::from_csv_line(&line) {
            Ok(password) => {
                if password.website.to_lowercase().contains(&query_lower)
                    || password.username.to_lowercase().contains(&query_lower)
                    || password.email.to_lowercase().contains(&query_lower)
                {
                    results.push((index + 1, password));
                }
            }
            Err(e) => {
                eprintln!("Warning: Skipping malformed line {}: {}", index + 1, e);
            }
        }
    }

    Ok(results)
}

fn list_passwords(limit: Option<usize>) -> Result<Vec<(usize, Password)>> {
    let file = open_for_reading()?;
    let reader = BufReader::new(file);
    let mut passwords = Vec::new();

    for (index, line_result) in reader.lines().enumerate() {
        if let Some(limit) = limit
            && index >= limit
        {
            break;
        }

        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }

        match Password::from_csv_line(&line) {
            Ok(password) => passwords.push((index + 1, password)),
            Err(e) => {
                eprintln!("Warning: Skipping malformed line {}: {}", index + 1, e);
            }
        }
    }

    Ok(passwords)
}

fn print_passwords(passwords: &[(usize, Password)]) {
    if passwords.is_empty() {
        println!("No passwords found.");
        return;
    }

    for (index, password) in passwords {
        println!("Index: {} - {}", index, password);
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    // Ensure the password file exists
    create_if_not_exists().map_err(|e| {
        eprintln!("Failed to initialize password file: {}", e);
        e
    })?;

    let cli = Cli::parse();

    match cli.command {
        Commands::List(args) => handle_list(args),
        Commands::Add(args) => handle_add(args),
        Commands::Delete(args) => handle_delete(args),
        Commands::Search(args) => handle_search(args),
    }
}

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

fn handle_add(args: AddArgs) -> Result<()> {
    let password = if args.interactive || are_any_fields_missing(&args) {
        collect_password_interactively(args)?
    } else {
        create_password_from_args(args)?
    };

    add_password(password)?;
    println!("✓ Password added successfully!");

    Ok(())
}

fn handle_delete(args: DeleteArgs) -> Result<()> {
    if args.index == 0 {
        return Err(PasswordManagerError::InvalidFormat(
            "Index must be greater than 0. Use 'crabpwd list' to see available indices."
                .to_string(),
        ));
    }

    // Show the password that will be deleted for confirmation
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

fn are_any_fields_missing(args: &AddArgs) -> bool {
    args.website.is_none()
        || args.username.is_none()
        || args.email.is_none()
        || args.password.is_none()
}

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

    let password = args.password.unwrap_or_else(|| {
        prompt_for_input("Password (required)", true).unwrap_or_else(|_| process::exit(1))
    });

    println!();
    Password::new(website, username, email, password)
}

fn prompt_for_input(prompt: &str, required: bool) -> io::Result<String> {
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

fn confirm_action(message: &str) -> io::Result<bool> {
    loop {
        print!("{} (y/N): ", message);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" | "" => return Ok(false),
            _ => println!("Please enter 'y' for yes or 'n' for no."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Test that CLI commands parse correctly
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
            "".to_string(),
            "user123".to_string(),
            "user@example.com".to_string(),
            "secret123".to_string(),
        );
        assert!(password.is_err());
    }

    #[test]
    fn test_csv_parsing() {
        let line = "github.com,user123,user@example.com,secret123";
        let password = Password::from_csv_line(line);
        assert!(password.is_ok());

        let password = password.unwrap();
        assert_eq!(password.website, "github.com");
        assert_eq!(password.username, "user123");
    }
}
