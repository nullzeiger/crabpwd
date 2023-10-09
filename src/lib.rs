// Copyright (c) Ivan Guerreschi. All rights reserved.
// Licensed under the MIT License. See LICENSE in the project root for license information.

pub mod file {
    use std::env;
    use std::fs;
    use std::fs::File;
    use std::io::Result;
    use std::path::Path;

    pub fn create() -> Result<()> {
        let file = filename();
        if !Path::new(&(file)).exists() {
            File::create(file)?;
        }
        Ok(())
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
