// Copyright (c) 2023 Ivan Guerreschi. All rights reserved.
// Licensed under the MIT License. See LICENSE in the project root for license information.

use clap::{Args, Parser, Subcommand};

use crabpwd::csv::{delete, new, print_all, search, Password};
use crabpwd::file::{create, open};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// All password
    All(AllArgs),
    /// Delete password
    Delete(DeleteArgs),
    /// New password
    New(NewArgs),
    /// Search password
    Search(SearchArgs),
}

#[derive(Args)]
struct AllArgs {
    #[arg(default_value_t = -1)]
    number: i32,
}

#[derive(Args)]
struct DeleteArgs {
    number: usize,
}

#[derive(Args)]
struct NewArgs {
    website: Option<String>,
    username: Option<String>,
    email: Option<String>,
    pwd: Option<String>,
}

#[derive(Args)]
struct SearchArgs {
    key: Option<String>,
}

fn main() {
    let create_file = create();
    match create_file {
        Ok(_) => println!("Created or existing file"),
        Err(error) => eprintln!("Error create file {:?}", error),
    }

    let cli = Cli::parse();

    match &cli.command {
        Commands::All(_number) => {
            let open_file = open();
            match open_file {
                Ok(file) => {
                    print_all(file, _number.number);
                }
                Err(error) => eprintln!("Error print all password: {:?}", error),
            }
        }
        Commands::Delete(number) => {
            let result_delete = delete(number.number);
            match result_delete {
                Ok(_) => println!("Delete password in position {}", number.number),
                Err(error) => {
                    eprintln!(
                        "Error to delete password in position {} {}",
                        number.number, error
                    )
                }
            }
        }
        Commands::New(password) => match password.website {
            Some(ref _password) => {
                let website = _password.to_owned();

                let username = match password.username {
                    Some(ref _username) => _username.to_owned(),
                    None => "*".to_string(),
                };

                let email = match password.email {
                    Some(ref _email) => _email.to_owned(),
                    None => "*".to_string(),
                };

                let pwd = match password.pwd {
                    Some(ref _pwd) => _pwd.to_owned(),
                    None => "*".to_string(),
                };

                let new_password = Password {
                    website: website.as_str(),
                    username: username.as_str(),
                    email: email.as_str(),
                    pwd: pwd.as_str(),
                };
                let create_password = new(new_password);
                match create_password {
                    Ok(_) => println!("Password create"),
                    Err(error) => eprintln!("Error new password not create {:?}", error),
                }
            }
            None => eprintln!("Insert correct value for new password"),
        },
        Commands::Search(key) => match key.key {
            Some(ref _key) => {
                let result_search = search(_key);
                match result_search {
                    Some(search) => println!("{}", search),
                    None => println!("Not found"),
                }
            }
            None => eprintln!("Enter a password search string"),
        },
    }
}
