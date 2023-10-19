// Copyright (c) Ivan Guerreschi. All rights reserved.
// Licensed under the MIT License. See LICENSE in the project root for license information.

pub mod csv {
    use std::fs::File;
    use std::io::{BufRead, BufReader, Result, Write};

    use crate::file::append;

    pub struct Password<'a> {
        website: &'a str,
        username: &'a str,
        email: &'a str,
        pwd: &'a str,
    }

    pub fn create(password: Password) -> Result<()> {
        if let Ok(mut file) = append() {
            let website = password.website;
            let username = password.username;
            let email = password.email;
            let pwd = password.email;
            let new_password = [website, username, email, pwd].join(",");
            file.write(new_password.as_bytes())?;
        }
        Ok(())
    }

    pub fn print_all(file: File) {
        let lines = BufReader::new(file).lines();
        for pwds in lines.skip(1) {
            if let Ok(pwd_line) = pwds {
                let pwd_values: Vec<&str> = pwd_line.split(',').collect();
                let password = Password {
                    website: pwd_values[0],
                    username: pwd_values[1],
                    email: pwd_values[2],
                    pwd: pwd_values[3],
                };
                println!(
                    "Website: {} Username: {} Email: {} Password: {}",
                    password.website, password.username, password.email, password.pwd
                );
            }
        }
    }
}

pub mod file {
    use std::env;
    use std::fs;
    use std::fs::{File, OpenOptions};
    use std::io::Result;
    use std::path::Path;

    pub fn append() -> Result<File> {
        let file = filename();
        let append_file = OpenOptions::new().append(true).open(file)?;
        Ok(append_file)
    }

    pub fn create() -> Result<()> {
        let file = filename();
        if !Path::new(&(file)).exists() {
            File::create(file)?;
        }
        Ok(())
    }

    pub fn open() -> Result<File> {
        let file = filename();
        let open_file = File::open(file)?;
        Ok(open_file)
    }

    pub fn delete() -> Result<()> {
        let file = filename();
        if Path::new(&(file)).exists() {
            fs::remove_file(file)?;
        }
        Ok(())
    }

    fn filename() -> String {
        let key = "HOME";
        let csv_file = "/.pwd.csv";
        let home = env::var(key).expect("$HOME is not set");
        home + csv_file
    }
}
