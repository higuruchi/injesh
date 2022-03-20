use crate::{container, image, image_downloader, user};
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
pub enum SubCommand<D>
where
    D: image_downloader::Downloader,
{
    Exec(Exec),
    Init(Init),
    Launch(Launch<D>),
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

impl Init {
    pub fn new() -> Result<Init, Box<dyn std::error::Error>> {
        let user = user::User::new()?;

        Ok(Init { user: user })
    }

    pub fn user(&self) -> &user::User {
        &self.user
    }
}

#[derive(Debug)]
pub struct Launch<D>
where
    D: image_downloader::Downloader,
{
    target_container: container::Container,
    rootfs_option: RootFSOption<D>,
    name: String,
    cmd: Cmd,
}

#[derive(Debug)]
pub enum RootFSOption<D>
where
    D: image_downloader::Downloader,
{
    Rootfs(PathBuf),
    RootfsImage(image::Image<D>),
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

impl<D> Launch<D>
where
    D: image_downloader::Downloader,
{
    pub fn new(
        target_container: container::Container,
        rootfs_option: RootFSOption<D>,
        name: String,
        cmd: Cmd,
    ) -> Result<Launch<D>, Box<dyn std::error::Error>> {
        Ok(Launch {
            target_container: target_container,
            rootfs_option: rootfs_option,
            name: name,
            cmd: cmd,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn target_container(&self) -> &container::Container {
        &self.target_container
    }

    pub fn rootfs_option(&self) -> &RootFSOption<D> {
        &self.rootfs_option
    }

    pub fn cmd(&self) -> &Cmd {
        &self.cmd
    }
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
                        Ok(user) => user.user().injesh_home().to_string(),
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

impl List {
    pub fn new() -> Result<List, Box<dyn std::error::Error>> {
        let user_info = user::User::new()?;

        Ok(List { user: user_info })
    }

    pub fn user(&self) -> &user::User {
        &self.user
    }
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

impl Exec {
    pub fn new(name: String, cmd: Option<String>) -> Exec {
        Exec {
            name: name,
            cmd: cmd,
        }
    }
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
        ContainerNotFound,
        MountFailed(nix::errno::Errno),
        UnmountFailed(nix::errno::Errno),
        CopyFailed(std::io::Error),
        OvarlayfsDirInvalid,
        RemoveFailed(std::io::Error),
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::NameNotFound => write!(f, "Name not found"),
                Error::ContainerNotFound => write!(f, "container not found"),
                Error::MountFailed(errno) | Error::UnmountFailed(errno) => {
                    let reason = match errno {
                        nix::errno::Errno::EPERM => "Operation not permitted".to_string(),
                        _ => format!("unknown error({})", errno.to_string()),
                    };
                    write!(f, "mount/umount failed: {}.", reason)
                }
                Error::CopyFailed(err) => write!(f, "copy failed: {}.", err),
                Error::OvarlayfsDirInvalid => write!(f, "ovarlayfs dir invalid"),
                Error::RemoveFailed(err) => write!(f, "remove failed: {}.", err),
            }
        }
    }

    impl std::error::Error for Error {}
}

impl Delete {
    pub fn new(name: String) -> Delete {
        Delete { name: name }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
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

impl File {
    pub fn new(name: String, from: PathBuf, to: PathBuf) -> File {
        File {
            name: name,
            from: from,
            to: to,
        }
    }
}

/// デバックコンテナ内で実行するコマンドを表す構造体
/// コンストラクタの引数として何も指定されていない場合は`/bin/bash`がデフォルトで用いられる
/// ```ignore
/// let cmd_vec = vec![
///     String::from("echo"),
///     String::from("hoge"),
/// ];
///
/// let cmd = Cmd::new(cmd_vec)
/// ```
#[derive(Debug)]
pub struct Cmd {
    /// mainはexecシステムコールの第1引数を表す。
    /// `echo hogehoge`の場合は`echo`が入る
    main: String,
    /// execシステムコールの第２引数以降が入る
    /// `echo hoge`の場合は`echo`, `hoge`が入る
    detail: Vec<String>,
}

impl Cmd {
    pub fn new(mut detail: Box<dyn Iterator<Item = String>>) -> Cmd {
        let main = match detail.next() {
            Some(cmd) => cmd.to_string(),
            None => "/bin/bash".to_string(),
        };

        let mut detail_vec = vec![main.clone()];
        for d in detail {
            detail_vec.push(d.to_string())
        }

        Cmd {
            main: main,
            detail: detail_vec,
        }
    }

    pub fn main(&self) -> &str {
        &self.main
    }

    pub fn detail(&self) -> &Vec<String> {
        &self.detail
    }

    pub fn detail_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &str> + 'a> {
        Box::new(self.detail.iter().map(|string| (*string).as_str()))
    }
}
