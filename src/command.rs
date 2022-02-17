// TODO::それぞれの方に応じたエラーを定義する
#[derive(Debug)]
pub enum Error {
    CommandError
}

#[derive(Debug)]
pub enum SubCommand {
    Exec(Exec),
    Init,
    Launch(Launch),
    List,
    Delete(String),
    File(FileSubCommand),
}

#[derive(Debug)]
pub struct Exec {
    name: String,
    cmd: Option<String>
}

#[derive(Debug)]
pub struct Launch {
    target_container: String,
    rootfs_option: RootFSOption,
    name: String,
    cmd: Option<String>
}

#[derive(Debug)]
pub enum RootFSOption {
    Rootfs(String),
    RootfsImage(String),
    RootfsDocker(String),
    RootfsLxd(String),
    None
}

#[derive(Debug)]
pub enum FileSubCommand {
    Pull(File),
    Push(File)
}

#[derive(Debug)]
pub struct File {
    name: String,
    from: String,
    to:   String
}

impl Launch {
    pub fn new(
        target_container: String,
        rootfs_option: RootFSOption,
        name: String,
        cmd: Option<String>
    ) -> Launch {
        Launch {
            target_container: target_container,
            rootfs_option: rootfs_option,
            name: name,
            cmd: cmd
        }
    }
}

impl Exec {
    pub fn new(
        name: String,
        cmd: Option<String>
    ) -> Exec {
        Exec {
            name: name,
            cmd: cmd
        }
    }
}

impl File {
    pub fn new(name: String, from: String, to: String) -> File {
        File{
            name: name,
            from: from,
            to: to
        }
    }
}