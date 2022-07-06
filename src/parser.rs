use crate::command::{
    self, Cmd, Delete, Error, Exec, File, FileSubCommand, Init, Launch, List, RootFSOption,
    SubCommand
};
use crate::{container, image, image_downloader, image_downloader_lxd, user, setting, setting_yaml};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

pub fn parse(
) -> Result<SubCommand<impl image_downloader::Downloader, impl setting::Reader + setting::Writer>, Box<dyn std::error::Error>> {
    let args: Cli = Cli::parse();
    match args.action {
        Action::Delete(delete) => Ok(SubCommand::Delete(initialize_delete(delete)?)),
        Action::Exec(exec) => Ok(SubCommand::Exec(initialize_exec(exec)?)),
        Action::File(file) => Ok(SubCommand::File(match file.action {
            FileAction::Pull(pull) => FileSubCommand::Pull(initialize_file_pull(pull)?),
            FileAction::Push(push) => FileSubCommand::Push(initialize_file_push(push)?),
        })),
        Action::Init => Ok(SubCommand::Init(initialize_init()?)),
        Action::Launch(launch) => Ok(SubCommand::Launch(initialize_launch(launch)?)),
        Action::List => Ok(SubCommand::List(initialize_list()?)),
    }
}

fn initialize_delete(delete: DeleteArgs) -> Result<Delete, Box<dyn std::error::Error>> {
    Ok(Delete::new(delete.name))
}

fn initialize_exec(exec: ExecArgs) -> Result<Exec, Box<dyn std::error::Error>> {
    Ok(Exec::new(exec.name, exec.cmd))
}

fn initialize_file_pull(pull: PullArgs) -> Result<File, Box<dyn std::error::Error>> {
    use command::file_error::Error;
    let name_and_path = parse_container_path(&pull.src).map_err(|_| Error::FromParseError)?;
    Ok(File::new(
        String::from(name_and_path.0),
        PathBuf::from(name_and_path.1),
        PathBuf::from(pull.dest),
    ))
}

fn initialize_file_push(push: PushArgs) -> Result<File, Box<dyn std::error::Error>> {
    use command::file_error::Error;
    let name_and_path = parse_container_path(&push.src).map_err(|_| Error::FromParseError)?;
    Ok(File::new(
        String::from(name_and_path.0),
        PathBuf::from(name_and_path.1),
        PathBuf::from(push.dest),
    ))
}

fn initialize_init() -> Result<Init, Box<dyn std::error::Error>> {
    Ok(Init::new()?)
}

fn initialize_launch(
    launch: LaunchArgs,
) -> Result<Launch<impl image_downloader::Downloader, impl setting::Reader + setting::Writer>, Box<dyn std::error::Error>> {

    let rootfs = check_rootfs(
        launch.opt_rootfs.as_ref().map(|r| r.as_str()),
        launch.opt_rootfs_image.as_ref().map(|r| r.as_str()),
        launch.opt_rootfs_docker.as_ref().map(|r| r.as_str()),
        launch.opt_rootfs_lxd.as_ref().map(|r| r.as_str()),
    )?;

    let container = container::Container::new(&launch.container_id_or_name)?;

    let user = user::User::new()?;
    let dcontainer_base = format!("{}/{}", user.containers(), launch.name);
    let setting_file_path = PathBuf::from(format!("{}/setting.yaml", &dcontainer_base));
    let setting_yaml_reader_writer = setting_yaml::YamlReaderWriter::new(&setting_file_path);

    Launch::new(
        container,
        rootfs,
        String::from(launch.name),
        Cmd::new(Box::new(launch.cmd.into_iter())),
        setting_yaml_reader_writer
    )
}

fn initialize_list() -> Result<List, Box<dyn std::error::Error>> {
    Ok(List::new()?)
}

fn check_rootfs(
    opt_rootfs: Option<&str>,
    opt_rootfs_image: Option<&str>,
    opt_rootfs_docker: Option<&str>,
    opt_rootfs_lxd: Option<&str>,
) -> Result<RootFSOption<impl image_downloader::Downloader>, Box<dyn std::error::Error>> {
    // must be one of the four options.
    let mut count_selected_opt: usize = 0;
    let mut rootfs = command::RootFSOption::None;

    if let Some(arg_rootfs) = opt_rootfs {
        count_selected_opt += 1;
        rootfs = RootFSOption::Rootfs(PathBuf::from(arg_rootfs));
    }
    if let Some(arg_rootfs_image) = opt_rootfs_image {
        count_selected_opt += 1;
        let user = user::User::new()?;

        // distribution/version format validation
        // e.g. busybox/1.34.1
        let distri_and_version = match arg_rootfs_image.split_once("/") {
            Some(d) => d,
            None => Err(crate::image::Error::ImageSyntaxError)?,
        };
        // .unwrap_or(Err(crate::image::Error::ImageSyntaxError)?);

        if distri_and_version.0.len() == 0 || distri_and_version.1.len() == 0 {
            Err(crate::image::Error::ImageSyntaxError)?
        }

        let download = image_downloader_lxd::Downloader::new(
            distri_and_version.0,
            distri_and_version.1,
            user.architecture(),
        )?;
        let image = image::Image::new(distri_and_version.0, distri_and_version.1, user, download)?;
        rootfs = RootFSOption::RootfsImage(image);
    }
    if let Some(arg_rootfs_docker) = opt_rootfs_docker {
        count_selected_opt += 1;
        rootfs = RootFSOption::RootfsDocker(arg_rootfs_docker.to_string());
    }
    if let Some(arg_rootfs_lxd) = opt_rootfs_lxd {
        count_selected_opt += 1;
        rootfs = RootFSOption::RootfsLxd(arg_rootfs_lxd.to_string());
    }

    if count_selected_opt > 1 {
        Err(Error::CommandError)?
    }

    Ok(rootfs)
}

// injesh file pull container_name:/path/to/src /path/to/dest
//                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ separate container_name and path
fn parse_container_path(
    name_colon_path_arg: &str,
) -> Result<(&str, &str), Box<dyn std::error::Error>> {
    // TODO: エラー型を精査する
    if !name_colon_path_arg.contains(':') {
        Err(crate::command::file_error::Error::FromParseError)?
    }

    let name_and_path = name_colon_path_arg
        .split_once(':')
        .ok_or(crate::command::file_error::Error::FromParseError)?;

    Ok(name_and_path)
}

const ABOUT_THIS_APP: &str =
    "Applications for debugging into containers without shells such as distroless and scratch. 
It is possible to enter a container by sharing namespaces \
such as cgroup, ipc, net, pid, user, uts, etc. with the container to be debugged. 
File operations performed in the debugging container do not affect the original container.";
const EXEC_ABOUT: &str = "Enter any existing container and run CMD.
If there is no CMD, invoke the shell in the configuration value file.";
const INIT_ABOUT: &str = "Initialize configuration files, .injesh directory, etc.
Run only once after installing injesh.";
const LAUNCH_ABOUT: &str = "Create a new debug container and get inside the debug container (mount overlayfs, etc.)
Get the executable files of the commands and dependent libraries described in the configuration file, create a rootfs, and then start it (pending).
NAME is the name of the debug container. If it is not specified, it will be generated automatically.
If CMD is not specified, the default shell is used.";
const DELETE_ABOUT: &str = "Remove the debug container";
const LIST_ABOUT: &str = "List debug containers";
const FILE_ABOUT: &str = "File operations in the debug container";
const FILE_PULL_ABOUT: &str = "Download the specified file of the debug container.";
const FILE_PUSH_ABOUT: &str =
    "Uploading the specified file of the host to the specified PATH of the debug container";

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

// basic arguments
#[derive(Parser)]
#[clap(about = ABOUT_THIS_APP, version = VERSION, author = AUTHOR)]
pub struct Cli {
    // subcommands
    // - delete
    // - exec
    // - file
    // - init
    // - launch
    // - list
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Subcommand)]
pub enum Action {
    // delete
    #[clap(name = "delete", about = DELETE_ABOUT)]
    Delete(DeleteArgs),
    // exec
    #[clap(name = "exec", about = EXEC_ABOUT)]
    Exec(ExecArgs),
    // file
    #[clap(name = "file", about = FILE_ABOUT)]
    File(FileArgs),
    // init
    #[clap(name = "init", about = INIT_ABOUT)]
    Init,
    // launch
    #[clap(name = "launch", about = LAUNCH_ABOUT)]
    Launch(LaunchArgs),
    // list
    #[clap(name = "list", about = LIST_ABOUT)]
    List,
}

#[derive(Args)]
pub struct DeleteArgs {
    #[clap()]
    pub name: String,
}

#[derive(Args)]
pub struct ExecArgs {
    #[clap()]
    pub name: String,
    #[clap()]
    pub cmd: Option<String>,
}

#[derive(Subcommand)]
pub enum FileAction {
    // pull
    #[clap(name = "pull", about = FILE_PULL_ABOUT)]
    Pull(PullArgs),
    // push
    #[clap(name = "push", about = FILE_PUSH_ABOUT)]
    Push(PushArgs),
}

#[derive(Args)]
pub struct FileArgs {
    // subcommands
    // - pull
    // - push
    #[clap(subcommand)]
    pub action: FileAction,
}

#[derive(Args)]
pub struct LaunchArgs {
    #[clap()]
    pub container_id_or_name: String,
    #[clap(long = "--rootfs")]
    pub opt_rootfs: Option<String>,
    #[clap(long = "--rootfs-image")]
    pub opt_rootfs_image: Option<String>,
    #[clap(long = "--rootfs-docker")]
    pub opt_rootfs_docker: Option<String>,
    #[clap(long = "--rootfs-lxd")]
    pub opt_rootfs_lxd: Option<String>,
    #[clap()]
    pub name: String,
    #[clap()]
    pub cmd: Vec<String>,
}

#[derive(Args)]
pub struct PullArgs {
    #[clap()]
    pub src: String,
    #[clap()]
    pub dest: String,
}

#[derive(Args)]
pub struct PushArgs {
    #[clap()]
    pub src: String,
    #[clap()]
    pub dest: String,
}
