use std::{env, error, fmt};

#[derive(Debug)]
pub struct User {
    injesh_home: String,
    images: String,
    containers: String,
    architecture: String,
}

#[derive(Debug)]
pub enum Error {
    HomeNotFound,
    UnsupportedArchitecture,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::HomeNotFound => write!(f, "Home not found!"),
            Error::UnsupportedArchitecture => write!(f, "cpu architecture unsupported"),
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

        let uname_machine = nix::sys::utsname::uname();
        // let uname_machine = uname_machine.machine();
        // TODO: add more architectures
        let architecture = match uname_machine.machine() {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            "armv7l" => "armhf",
            _ => {
                Err(Error::UnsupportedArchitecture)?
            }
        };

        Ok(User {
            injesh_home: format!("{}", &injesh_homedir),
            images: format!("{}/images", &injesh_homedir),
            containers: format!("{}/containers", &injesh_homedir),
            architecture: format!("{}", architecture),
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
    pub fn architecture(&self) -> &str {
        &self.architecture
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
    #[test]
    fn test_user_architecture() {
        let userinfo = User::new();
        assert_eq!(userinfo.unwrap().architecture(), "amd64");
    }
}
