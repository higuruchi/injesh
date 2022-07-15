use std::{env, error, fmt};

#[derive(Debug)]
pub enum Error {
    NotImplemented,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotImplemented => write!(f, "user::macos: Not Implemented"),
        }
    }
}

impl error::Error for Error {}

#[cfg(target_os = "macos")]
pub fn injesh_home_dir() -> Result<String, Box<dyn std::error::Error>> {
    Err(Error::NotImplemented)
}
