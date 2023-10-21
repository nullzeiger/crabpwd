// Copyright (c) Ivan Guerreschi. All rights reserved.
// Licensed under the MIT License. See LICENSE in the project root for license information.

pub mod csv {
    use std::fs::File;
    use std::io::{BufRead, BufReader, Result, Write};
    use std::ops::Add;

    use crate::file::append;

    pub struct Password<'a> {
        pub website: &'a str,
        pub username: &'a str,
        pub email: &'a str,
        pub pwd: &'a str,
    }

    pub fn new(password: Password) -> Result<()> {
        let append_file = append();
        match append_file {
            Ok(mut file) => {
                let website = password.website;
                let username = password.username;
                let email = password.email;
                let pwd = password.email;
                let new_password = [website, username, email, pwd].join(",").add("\n");
                file.write_all(new_password.as_bytes())?;
            }
            Err(error) => panic!("Error append file {:?}", error),
        }
        Ok(())
    }

    pub fn print_all(file: File) {
        let lines = BufReader::new(file).lines();
        let mut i = 0;
        for pwds in lines.skip(1) {
            let pwd_line = pwds;
            match pwd_line {
                Ok(line) => {
                    let pwd_values: Vec<&str> = line.split(',').collect();
                    if pwd_values.len() == 4 {
                        let password = Password {
                            website: pwd_values[0],
                            username: pwd_values[1],
                            email: pwd_values[2],
                            pwd: pwd_values[3],
                        };

                        i += 1;
                        println!(
                            "Index: {} Website: {} Username: {} Email: {} Password: {}",
                            i, password.website, password.username, password.email, password.pwd
                        );
                    } else {
                        eprintln!(".pwd.csv document not formatted correctly in this line");
                    }
                }
                Err(error) => panic!("Error read file lines {:?}", error),
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
        let csv_file = "/.pwdbk.csv";
        let home = env::var(key).expect("$HOME is not set");
        home + csv_file
    }
}
