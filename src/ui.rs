use crate::models::Password;

/// Prints a single password entry with its 1-based index.
pub fn print_password(index: usize, pwd: &Password) {
    println!(
        "[{}] {} | {} | {} | {}",
        index, pwd.website, pwd.username, pwd.email, pwd.password
    );
}

/// Prints a list of (index, Password) pairs.
pub fn print_password_list(list: &[(usize, Password)]) {
    if list.is_empty() {
        println!("No passwords found.");
        return;
    }
    for (i, pwd) in list {
        print_password(*i, pwd);
    }
}
