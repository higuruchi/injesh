use std::{env, error, fmt};

#[derive(Debug)]
pub struct User {
    injesh_home: String,
    images: String,
    containers: String,
}

#[derive(Debug)]
pub enum Error {
    HomeNotFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::HomeNotFound => write!(f, "Home not found!"),
        }
    }
}

impl error::Error for Error {}

impl User {
    pub fn new() -> Result<User, Box<dyn std::error::Error>> {
        let injesh_homedir = match env::var("HOME") {
            Ok(home) => home + "/.injesh",
            Err(_) => return Err(Error::HomeNotFound)?,
        };

        Ok(User {
            injesh_home: format!("{}", &injesh_homedir),
            images: format!("{}/images", &injesh_homedir),
            containers: format!("{}/containers", &injesh_homedir),
        })
    }

    pub fn injesh_home(&self) -> &str {
        &self.injesh_home
    }
    pub fn images(&self) -> &str {
        &self.images
    }
    pub fn containers(&self) -> &str {
        &self.containers
    }
}

mod tests {
    use super::*;
    #[test]
    fn test_user_home() {
        let userinfo = User::new();
        assert_eq!(userinfo.unwrap().injesh_home(), "/home/runner/.injesh");
    }
    #[test]
    fn test_user_images() {
        let userinfo = User::new();
        assert_eq!(userinfo.unwrap().images(), "/home/runner/.injesh/images");
    }
    #[test]
    fn test_user_containers() {
        let userinfo = User::new();
        assert_eq!(
            userinfo.unwrap().containers(),
            "/home/runner/.injesh/containers"
        );
    }
}
