// Copyright (c) 2023 Ivan Guerreschi. All rights reserved.
// Licensed under the MIT License. See LICENSE in the project root for license information.

pub mod csv {
    use crate::file::{append, delete_file, filename, open, rename};
    use std::fs::{self, File};
    use std::io::{BufRead, BufReader, Result, Write};
    use std::ops::Add;

    pub struct Password<'a> {
        pub website: &'a str,
        pub username: &'a str,
        pub email: &'a str,
        pub pwd: &'a str,
    }

    pub fn delete(key: usize) -> Result<()> {
        let open_file = open();
        match open_file {
            Ok(file) => {
                let lines = BufReader::new(file).lines();
                let mut pwd = Vec::new();
                for (i, pwds) in lines.enumerate() {
                    match pwds {
                        Ok(line) => {
                            if i + 1 != key {
                                pwd.push(line);
                            }
                        }
                        Err(error) => panic!("Error read file liner {:?}", error),
                    }
                }
                let file_tmp = filename("/.pwd_tmp.csv");
                fs::write(file_tmp, pwd.join("\n"))?;
                let delete_original_file = delete_file();
                match delete_original_file {
                    Ok(_) => println!("File deleted"),
                    Err(error) => panic!("Error delete file {:?}", error),
                }
                let rename_file = rename();
                match rename_file {
                    Ok(_) => println!("File rename"),
                    Err(error) => panic!("Error rename file {:?}", error),
                }
            }
            Err(error) => panic!("Error open file {:?}", error),
        }
        Ok(())
    }

    pub fn search(key: &str) -> Option<String> {
        let open_file = open();
        match open_file {
            Ok(file) => {
                let lines = BufReader::new(file).lines();
                for pwds in lines {
                    match pwds {
                        Ok(line) => {
                            if let Some(result) = line.find(key) {
                                if let Some(line_result) = line.get(result..) {
                                    return Some(line_result.to_string());
                                }
                            }
                        }
                        Err(error) => panic!("Error read file lines {:?}", error),
                    }
                }
            }
            Err(error) => panic!("Error open file: {:?}", error),
        }
        None
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

    pub fn print_all(file: File, number_row: i32) {
        let lines = BufReader::new(file).lines();
        let mut i = 0;
        for pwds in lines {
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
            if number_row == -1 {
                continue;
            } else if i > number_row - 1 {
                break;
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

    const CSV_FILE: &str = "/.pwd.csv";

    pub fn append() -> Result<File> {
        let file = filename(CSV_FILE);
        let append_file = OpenOptions::new().append(true).open(file)?;
        Ok(append_file)
    }

    pub fn create() -> Result<()> {
        let file = filename(CSV_FILE);
        if !Path::new(&(file)).exists() {
            File::create(file)?;
        }
        Ok(())
    }

    pub fn open() -> Result<File> {
        let file = filename(CSV_FILE);
        let open_file = File::open(file)?;
        Ok(open_file)
    }

    pub fn delete_file() -> Result<()> {
        let file = filename(CSV_FILE);
        if Path::new(&(file)).exists() {
            fs::remove_file(file)?;
        }
        Ok(())
    }

    pub fn rename() -> Result<()> {
        fs::rename(filename("/.pwd_tmp.csv"), filename(CSV_FILE))?;
        Ok(())
    }

    pub fn filename(file: &str) -> String {
        let key = "HOME";
        match env::var(key) {
            Ok(home) => home + file,
            Err(error) => panic!("${}, is not set {}", key, error),
        }
    }
}
