// Copyright (c) 2023 Ivan Guerreschi. All rights reserved.
// Licensed under the MIT License. See LICENSE in the project root for license information.

use std::env;
use std::io;

use crabpwd::csv::{new, print_all, search, Password};
use crabpwd::file::{create, open};

fn help() {
    println!(
        "
        crabpwd Password manager\n
        a    Print all password
        s    <String>  Search password
        n    Create new password
        "
    );
}

fn main() {
    let create_file = create();
    match create_file {
        Ok(_) => println!("File create"),
        Err(error) => eprintln!("Error create file {:?}", error),
    }

    let args: Vec<String> = env::args().collect();

    help();

    match args.len() {
        1 => {
            println!("No arguments passed");
        }

        2 => {
            let cmd = &args[1];
            if cmd == "a" {
                let open_file = open();
                match open_file {
                    Ok(file) => {
                        print_all(file);
                    }
                    Err(error) => eprintln!("Error print all password: {:?}", error),
                }
            } else if cmd == "n" {
                let mut website = String::new();
                let mut username = String::new();
                let mut email = String::new();
                let mut pwd = String::new();

                println!("Website");
                io::stdin()
                    .read_line(&mut website)
                    .expect("Failed to read line");
                website = website.trim().to_string();

                println!("Username");
                io::stdin()
                    .read_line(&mut username)
                    .expect("Enter Username");
                username = username.trim().to_string();

                println!("Email");
                io::stdin()
                    .read_line(&mut email)
                    .expect("Failed to read line");
                email = email.trim().to_string();

                println!("Password");
                io::stdin()
                    .read_line(&mut pwd)
                    .expect("Failed to read line");
                pwd = pwd.trim().to_string();

                let new_password = Password {
                    website: &website,
                    username: &username,
                    email: &email,
                    pwd: &pwd,
                };

                let create_password = new(new_password);
                match create_password {
                    Ok(_) => println!("Password create"),
                    Err(error) => eprintln!("Error new password not create {:?}", error),
                }
            } else {
                eprintln!("error: invalid command");
                help();
            }
        }

        3 => {
            let cmd = &args[1];
            let key = &args[2];

            if cmd == "s" {
                let result_search = search(key);
                match result_search {
                    Some(search) => println!("{}", search),
                    None => println!("Not found"),
                }
            } else {
                eprintln!("Error invalid command");
                help();
            }
        }
        _ => {
            help();
        }
    }
}
