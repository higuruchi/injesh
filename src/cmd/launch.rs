use crate::command::{self, RootFSOption};
use crate::image_downloader::Downloader;
use crate::{namespace, setting, user, utils};
use std::path::{Path, PathBuf};
use std::{error, fmt, fs::create_dir_all};

use nix::mount::{mount, MsFlags};
use nix::sched::{unshare, CloneFlags};
use nix::sys::wait::waitpid;
use nix::unistd::{chdir, chroot, fork, ForkResult, Gid, Uid};

#[derive(Debug)]
pub enum Error {
    AlreadyExists,
    Umount,
    NonValidUnicode,
    InvalidRootFSPath,
    NotImplemented,
    Fork,
    Waitpid,
    UnmountFailed(nix::errno::Errno),
    MountFailed(nix::errno::Errno),
    InputValue,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::AlreadyExists => write!(f, "Debug Container Already Exists"),
            Error::Umount => write!(f, "umount error"),
            Error::NonValidUnicode => write!(f, "non valid unicode"),
            Error::InvalidRootFSPath => write!(f, "invalid rootfs path"),
            Error::NotImplemented => write!(f, "Not implemented"),
            Error::Fork => write!(f, "failed fork"),
            Error::Waitpid => write!(f, "failed waitpid"),
            Error::InputValue => write!(f, "Input value is illegal"),
            Error::UnmountFailed(e) => write!(f, "Unmount failed due to {}", e),
            Error::MountFailed(e) => write!(f, "Mount failed due to {}", e),
        }
    }
}

impl error::Error for Error {}

pub struct LaunchStruct;

impl LaunchStruct {
    /// rootfsの種類などが記載された設定ファイルsetting.yamlを~/.injesh/containers/に作成する
    /// /var/lib/docker/overlay2/<HASH_ID>/upperを~/.injesh/containers/<hoge>/upperに対してコピーする
    /// デバック対象コンテナのlowerdirに対してrootfsを追加した後reloadする
    /// デバック対象コンテナのプロセスIDを取得
    /// forkする
    /// 取得したデバック対象コンテナプロセスIDをもとにsetnsをし、名前空間を同一にする
    /// 与えられた初期実行ファイルをexecする
    pub fn launch<DO: Downloader, RW: setting::Reader + setting::Writer>(
        &self,
        launch: &mut command::Launch<DO, RW>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // injeshコマンドが初期化されてるかどうかチェック
        utils::check_initialized()?;

        // デバックコンテナの設定ファイル、ディレクトリ周りの初期化
        initialize_setting(launch)?;

        rootfs_injected_overlayfs_mount(launch)?;

        // デバック対象コンテナのプロセスIDとネームスペースのファイルディスクリプタを取得
        let container_pid = launch.target_container().pid();
        let ns = namespace::Ns::new(container_pid)?;
        // let gid = Gid::current();
        // let uid = Uid::current();

        // setnsで名前空間を変更
        ns.setns_net()?;
        ns.setns_cgroup()?;
        ns.setns_ipc()?;
        ns.setns_pid()?;
        ns.setns_uts()?;
        unshare(CloneFlags::CLONE_NEWNS)?;
        // unshare(CloneFlags::CLONE_NEWUSER)?;
        // common::new_uidmap(&uid)?;
        // common::new_gidmap(&gid)?;


        unsafe {
            match fork() {
                // 親プロセスの場合
                Ok(ForkResult::Parent { child, .. }) => match waitpid(child, None) {
                    Ok(status) => println!("Child {:?}", status),
                    Err(_) => Err(Error::Waitpid)?,
                },
                // 子プロセス
                Ok(ForkResult::Child) => {

                    let user = user::User::new()?;
                    let dcontainer_base = format!("{}/{}", user.containers(), launch.name());
                    let dcontainer_base_merged = format!("{}/merged", &dcontainer_base);

                    chroot(&PathBuf::from(&dcontainer_base_merged))?;
                    chdir("/")?;
                    mount(Some("proc"), "/proc", Some("proc"), MsFlags::empty(), None::<&Path>).map_err(|why| Error::MountFailed(why))?;

                    // execでプログラムを実行
                    use std::os::unix::process::CommandExt;
                    std::process::Command::new(launch.cmd().main())
                        .args(launch.cmd().detail())
                        .exec();
                }
                Err(_) => return Err(Error::Fork)?,
            }
        };

        Ok(())
    }

    pub fn new() -> LaunchStruct {
        LaunchStruct
    }
}

/// デバックコンテナを起動するために必要なディレクトリ群を初期化
///
/// - `~/.injesh/containers/<CONTAINER_NAME>/upper`
///
///     デバックコンテナ起動前の`/var/lib/docker/overlay2/<HASH_ID>/upper`を保存しておくためのディレクトリ
///
/// - `~/.injesh/containers/<CONTAINER_NAME>/setting.yaml`
///
///     デバックコンテナの`rootfs`やデバックコンテナ作成後に実行するコマンドなどを保存する設定ファイル
fn initialize_setting<DO: Downloader, RW: setting::Reader + setting::Writer>(
    launch: &mut command::Launch<DO, RW>,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = user::User::new()?;
    let dcontainer_base = format!("{}/{}", user.containers(), launch.name());

    // rootfsの種類などが記載された設定ファイルsetting.yamlを~/.injesh/containers/に作成する
    if Path::new(&dcontainer_base).exists() {
        Err(Error::AlreadyExists)?
    }

    create_dir_all(format!("{}/upper/proc", &dcontainer_base))?;
    create_dir_all(format!("{}/merged", &dcontainer_base))?;
    create_dir_all(format!("{}/worker", &dcontainer_base))?;

    let target_container_id = launch.target_container().container_id().to_string();
    launch
        .setting_handler_mut()
        .init(&target_container_id, setting::Shell::Bash, &[]);
    launch.setting_handler().write()?;

    Ok(())
}

/// rootfsを挿入したoverlayfsをマウントする
/// mountpoint: `~/.injesh/containers/<CONTAINER_NAME>/merged`
fn rootfs_injected_overlayfs_mount<DO: Downloader, RW: setting::Reader + setting::Writer>(
    launch: &command::Launch<DO, RW>,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = user::User::new()?;
    let dcontainer_base = format!("{}/{}", user.containers(), launch.name());

    let rootfs_path = match launch.rootfs_option() {
        RootFSOption::RootfsImage(image) => {
            match image.check_rootfs_newest() {
                Ok(is_newest) => {
                    if !is_newest {
                        image.download_image()?;
                    }
                }
                Err(e) => {
                    Err(e)?;
                }
            }
            image.rootfs_path()
        }
        _ => Err(Error::NotImplemented)?,
    };

    let dcontainer_merged = PathBuf::from(format!("{}/merged", &dcontainer_base));
    let dcontainer_worker = format!("{}/worker", &dcontainer_base);
    let dcontainer_upper = format!("{}/upper", &dcontainer_base);

    let target_container_merged = launch
        .target_container()
        .mergeddir()
        .to_str()
        .ok_or(Error::NonValidUnicode)?;

    let mount_data = format!(
        "lowerdir={}:{},upperdir={},workdir={}",
        rootfs_path.to_str().ok_or(Error::InvalidRootFSPath)?,
        target_container_merged,
        dcontainer_upper,
        dcontainer_worker
    );
    mount(
        Some("overlay"),
        &dcontainer_merged,
        Some("overlay"),
        MsFlags::empty(),
        Some(mount_data.as_str()),
    )
    .map_err(|why| Error::MountFailed(why))?;

    Ok(())
}
