use std::{env, fmt, error};

#[derive(Debug)]
pub struct User {
    home: String
}

#[derive(Debug)]
pub enum Error {
    HomeNotFound
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::HomeNotFound => write!(f, "Home not found!")
        }
    }
}

impl error::Error for Error {}

impl User {
    pub fn new() -> Result<User, Box<dyn std::error::Error>> {
        let home = match env::var("HOME") {
            Ok(home) => home,
            Err(_) => return Err(Error::HomeNotFound)?
        };

        Ok(User{
            home: home
        })
    }

    pub fn home(&self) -> &str {
        &self.home
    }
}