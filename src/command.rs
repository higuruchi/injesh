use crate::user;
use std::fmt;
use std::path::PathBuf;

// TODO::それぞれの方に応じたエラーを定義する
#[derive(Debug)]
pub enum Error {
    CommandError,
    NotInitialized,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError => write!(f, "Sub Command Error"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum SubCommand {
    Exec(Exec),
    Init(Init),
    Launch(Launch),
    List(List),
    Delete(Delete),
    File(FileSubCommand),
}

#[derive(Debug)]
pub struct Init {
    user: user::User,
}

pub mod init_error {
    use std::fmt;

    #[derive(Debug)]
    pub enum Error {
        AlreadyInitialized,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::AlreadyInitialized => write!(f, "Already Initialized!"),
            }
        }
    }

    impl std::error::Error for Error {}
}

#[derive(Debug)]
pub struct Exec {
    name: String,
    cmd: Option<String>,
}

pub mod exec_error {
    use std::fmt;

    #[derive(Debug)]
    pub enum Error {
        NameNotFound,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::NameNotFound => write!(f, "Name not Found"),
            }
        }
    }

    impl std::error::Error for Error {}
}

#[derive(Debug)]
pub struct Launch {
    target_container: String,
    rootfs_option: RootFSOption,
    name: String,
    cmd: Option<String>,
}

#[derive(Debug)]
pub struct List {
    user: user::User,
}

pub mod list_error {
    use crate::command::List;
    use std::fmt;

    #[derive(Debug)]
    pub enum Error {
        // failed to read directory
        ReadDirError(std::io::Error),
        // no containers found
        NoContainers,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::ReadDirError(err) => {
                    let injesh_home = match List::new() {
                        Ok(user) => user.get_user().injesh_home().to_string(),
                        Err(_) => "[injesh_home]".to_string(),
                    };
                    write!(f, "Failed to reading {}: {}.", injesh_home, err)
                }
                Error::NoContainers => write!(f, "No Containers Found"),
            }
        }
    }

    impl std::error::Error for Error {}
}

#[derive(Debug)]
pub enum RootFSOption {
    Rootfs(PathBuf),
    RootfsImage(String),
    RootfsDocker(String),
    RootfsLxd(String),
    None,
}

pub mod launch_error {
    use std::fmt;

    #[derive(Debug)]
    pub enum Error {
        ContainerIdOrNameNotFound,
        NameNotFound,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::ContainerIdOrNameNotFound => write!(f, "Container id or name not found"),
                Error::NameNotFound => write!(f, "Name not found"),
            }
        }
    }

    impl std::error::Error for Error {}
}

#[derive(Debug)]
pub struct Delete {
    name: String,
}

pub mod delete_error {
    use std::fmt;

    #[derive(Debug)]
    pub enum Error {
        NameNotFound,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::NameNotFound => write!(f, "Name not found"),
            }
        }
    }

    impl std::error::Error for Error {}
}

#[derive(Debug)]
pub enum FileSubCommand {
    Pull(File),
    Push(File),
}

#[derive(Debug)]
pub struct File {
    name: String,
    from: PathBuf,
    to: PathBuf,
}

pub mod file_error {
    use std::fmt;

    #[derive(Debug)]
    pub enum Error {
        FileOperationNotFound,
        FromParseError,
        FromNotFound,
        ToNotFound,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::FileOperationNotFound => write!(f, "File operation not found"),
                Error::FromParseError => write!(f, "From parse error"),
                Error::FromNotFound => write!(f, "From not found"),
                Error::ToNotFound => write!(f, "To not found"),
            }
        }
    }

    impl std::error::Error for Error {}
}

impl Init {
    pub fn new() -> Result<Init, Box<dyn std::error::Error>> {
        let user = user::User::new()?;

        Ok(Init { user: user })
    }

    pub fn user(&self) -> &user::User {
        &self.user
    }
}

impl Launch {
    pub fn new(
        target_container: String,
        rootfs_option: RootFSOption,
        name: String,
        cmd: Option<String>,
    ) -> Launch {
        Launch {
            target_container: target_container,
            rootfs_option: rootfs_option,
            name: name,
            cmd: cmd,
        }
    }
}

impl List {
    pub fn new() -> Result<List, Box<dyn std::error::Error>> {
        let user_info = user::User::new()?;

        Ok(List { user: user_info })
    }

    pub fn get_user(&self) -> &user::User {
        &self.user
    }
}

impl Exec {
    pub fn new(name: String, cmd: Option<String>) -> Exec {
        Exec {
            name: name,
            cmd: cmd,
        }
    }
}

impl Delete {
    pub fn new(name: String) -> Delete {
        Delete { name: name }
    }
}

impl File {
    pub fn new(name: String, from: PathBuf, to: PathBuf) -> File {
        File {
            name: name,
            from: from,
            to: to,
        }
    }
}
