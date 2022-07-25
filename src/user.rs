mod linux;
mod macos;
mod windows;

use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    WindowsHomePathUnimplemented,
    MacOSHomePathUnimplemented,
    SudoUserNotFound,
    HomeNotFound,
    UnsupportedArchitecture,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::WindowsHomePathUnimplemented => {
                write!(f, "user: windows home path unimplemented")
            }
            Error::MacOSHomePathUnimplemented => write!(f, "user: macos home path unimplemented"),
            Error::SudoUserNotFound => write!(f, "user: sudo user not found"),
            Error::HomeNotFound => write!(f, "user: Home not found"),
            Error::UnsupportedArchitecture => write!(f, "user: unsupported architecture"),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug)]
pub struct User {
    injesh_home: String,
    images: String,
    containers: String,
    architecture: CpuArchitecture,
}

impl User {
    pub fn new() -> Result<User, Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        let injesh_homedir = linux::injesh_home_dir()?;

        #[cfg(target_os = "windows")]
        let injesh_homedir = windows::home_dir()?;

        #[cfg(target_os = "macos")]
        let injesh_homedir = macos::home_dir()?;

        let architecture = CpuArchitecture::new()?;

        Ok(User {
            injesh_home: format!("{}", &injesh_homedir),
            images: format!("{}/images", &injesh_homedir),
            containers: format!("{}/containers", &injesh_homedir),
            architecture,
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
    pub fn architecture(&self) -> CpuArchitecture {
        self.architecture
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CpuArchitecture {
    Aarch64,
    Amd64,
    Armhf,
}

impl fmt::Display for CpuArchitecture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CpuArchitecture::Aarch64 => write!(f, "aarch64"),
            CpuArchitecture::Amd64 => write!(f, "amd64"),
            CpuArchitecture::Armhf => write!(f, "armhf"),
        }
    }
}

impl CpuArchitecture {
    fn new() -> Result<CpuArchitecture, Box<dyn std::error::Error>> {
        let uname = nix::sys::utsname::uname();
        // TODO: add more architectures
        let architecture = match uname.machine() {
            "x86_64" => CpuArchitecture::Amd64,
            "aarch64" => CpuArchitecture::Aarch64,
            "armv7l" => CpuArchitecture::Armhf,
            _ => Err(Error::UnsupportedArchitecture)?,
        };

        Ok(architecture)
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
        assert_eq!(userinfo.unwrap().architecture(), CpuArchitecture::Amd64);
    }
    #[test]
    fn test_user_architecture_display() {
        let userinfo = User::new();
        assert_eq!(format!("{}", userinfo.unwrap().architecture()), "amd64");
    }
}
