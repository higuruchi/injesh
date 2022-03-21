use crate::command::{self, RootFSOption};
use crate::image_downloader::Downloader;
use crate::launch::Launch;
use crate::user;
use crate::utils;
use std::ffi::{CString, OsStr};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::{error, fmt, fs};

use nix::mount::{mount, umount2, MntFlags, MsFlags};
use nix::sched::{setns, CloneFlags};
use nix::sys::wait::waitpid;
use nix::unistd::{execv, fork, ForkResult};

use std::os::unix::io::AsRawFd;

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
            AlreadyExists => write!(f, "Debug Container Already Exists"),
            Umount => write!(f, "umount error"),
            NonValidUnicode => write!(f, "non valid unicode"),
            InvalidRootFSPath => write!(f, "invalid rootfs path"),
            NotImplemented => write!(f, "Not implemented"),
            Fork => write!(f, "failed fork"),
            Waitpid => write!(f, "failed waitpid"),
            InputValue => write!(f, "Input value is illegal"),
        }
    }
}

impl error::Error for Error {}

pub struct LaunchStruct;

impl<DO> Launch<DO> for LaunchStruct
where
    DO: Downloader,
{
    /// rootfsの種類などが記載された設定ファイルsetting.yamlを~/.injesh/containers/に作成する
    /// /var/lib/docker/overlay2/<HASH_ID>/upperを~/.injesh/containers/<hoge>/upperに対してコピーする
    /// デバック対象コンテナのlowerdirに対してrootfsを追加した後reloadする
    /// デバック対象コンテナのプロセスIDを取得
    /// forkする
    /// 取得したデバック対象コンテナプロセスIDをもとにsetnsをし、名前空間を同一にする
    /// 与えられた初期実行ファイルをexecする
    fn launch(&self, launch: &mut command::Launch<DO>) -> Result<(), Box<dyn std::error::Error>> {
        // injeshコマンドが初期化されてるかどうかチェック
        utils::check_initialized()?;

        // デバックコンテナの設定ファイル、ディレクトリ周りの初期化
        initialize_setting(launch)?;

        // overlyafsのマウントし直し
        remount(launch)?;

        // デバック対象コンテナのlowerdirに対してrootfsを追加した後reloadする
        launch.target_container().restart()?;
        launch.target_container_mut().update_pid()?;

        // デバック対象コンテナのプロセスIDとネームスペースのファイルディスクリプタを取得
        let container_pid = launch.target_container().pid();
        let ns = Ns::new(container_pid)?;

        unsafe {
            match fork() {
                // 親プロセスの場合
                Ok(ForkResult::Parent { child, .. }) => match waitpid(child, None) {
                    Ok(status) => println!("Child exited {:?}", status),
                    Err(_) => Err(Error::Waitpid)?,
                },
                // 子プロセス
                Ok(ForkResult::Child) => {
                    // setnsで名前空間を変更
                    ns.setns_net()?;
                    ns.setns_cgroup()?;
                    ns.setns_ipc()?;
                    ns.setns_pid()?;

                    // これがあるとなぜか失敗する
                    // setns(user_fd.as_raw_fd(), CloneFlags::empty())?;

                    ns.setns_mnt()?;
                    ns.setns_uts()?;

                    // execでプログラムを実行

                    let main = CString::new(launch.cmd().main())?;

                    let detail: Vec<CString> = launch
                        .cmd()
                        .detail_iter()
                        .filter_map(|s| CString::new(s).ok())
                        .collect();

                    execv(&main, &detail)?;
                }
                Err(_) => return Err(Error::Fork)?,
            }
        };

        Ok(())
    }
}

impl LaunchStruct {
    pub fn new<DO>() -> impl Launch<DO>
    where
        DO: Downloader,
    {
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
fn initialize_setting<DO: Downloader>(
    launch: &command::Launch<DO>,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = user::User::new()?;
    let dcontainer_base = format!("{}/{}", user.containers(), launch.name());

    // rootfsの種類などが記載された設定ファイルsetting.yamlを~/.injesh/containers/に作成する
    if Path::new(&dcontainer_base).exists() {
        Err(Error::AlreadyExists)?
    }

    fs::create_dir_all(format!("{}/upper", &dcontainer_base))?;
    let mut setting_file = fs::File::create(format!("{}/setting.yaml", &dcontainer_base))?;
    // TODO: 設定ファイルの形式を決めてない
    setting_file.write_all(b"content of setting.yaml")?;

    // /var/lib/docker/overlay2/<HASH_ID>/upperを~/.injesh/containers/<hoge>/upperに対してコピーする
    copy_dir_recursively(
        launch.target_container().upperdir(),
        format!("{}/upper", &dcontainer_base),
    )?;

    Ok(())
}

/// ディレクトリを再起的にコピーする関数
/// ```ignore
/// copy_dir_recursively("/path/to/from", "/path/to/to")
/// ```
fn copy_dir_recursively<P, Q>(from: P, to: Q) -> Result<(), Box<dyn std::error::Error>>
where
    P: AsRef<OsStr> + AsRef<Path>,
    Q: AsRef<OsStr> + AsRef<Path>,
{
    if !Path::new(&from).is_dir() {
        Err(Error::InputValue)?
    }
    if !Path::new(&to).is_dir() {
        Err(Error::InputValue)?
    }

    let to_name = Path::new(&to).to_str().ok_or(Error::InputValue)?;
    for entry_result in fs::read_dir(from)? {
        let entry = entry_result?;
        let to_path = Path::new(to_name).join(entry.path().file_name().ok_or(Error::InputValue)?);

        if entry.file_type()?.is_dir() {
            fs::create_dir(&to_path)?;
            copy_dir_recursively(entry.path(), &to_path)?;
        } else {
            fs::copy(entry.path(), &to_path)?;
        }
    }
    Ok(())
}

/// `/var/lib/docker/overlay2/<HASH_ID>/upper`を`unmount`した後、任意の`rootfs`を挿入したものを`mount`する
fn remount<DO: Downloader>(launch: &command::Launch<DO>) -> Result<(), Box<dyn std::error::Error>> {
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
    let lower_dir = launch
        .target_container()
        .lowerdir()
        .to_str()
        .ok_or(Error::NonValidUnicode)?;
    let upper_dir = launch
        .target_container()
        .upperdir()
        .to_str()
        .ok_or(Error::NonValidUnicode)?;
    let work_dir = launch
        .target_container()
        .workdir()
        .to_str()
        .ok_or(Error::NonValidUnicode)?;

    // デバック対象コンテナのlowerdirに対してrootfsを追加した後reloadする
    umount2(launch.target_container().mergeddir(), MntFlags::empty())
        .map_err(|why| Error::UnmountFailed(why))?;

    mount(
        Some("overlay"),
        // launch.target_container().mergeddir(),
        launch.target_container().mergeddir(),
        Some("overlay"),
        MsFlags::empty(),
        Some(
            format!(
                "lowerdir={}:{},upperdir={},workdir={}",
                rootfs_path.to_str().ok_or(Error::InvalidRootFSPath)?,
                lower_dir,
                upper_dir,
                work_dir,
            )
            .as_str(),
        ),
    )
    .map_err(|why| Error::MountFailed(why))?;

    Ok(())
}

/// プロセスのnamespaceファイルディスクリプタを管理する構造体
struct Ns {
    net: File,
    cgroup: File,
    ipc: File,
    pid: File,
    user: File,
    mnt: File,
    uts: File,
}

impl Ns {
    fn new(container_pid: u32) -> Result<Ns, Box<dyn std::error::Error>> {
        let ns_base_path = format!("/proc/{}/ns", container_pid);

        Ok(Ns {
            net: File::open(format!("{}/net", &ns_base_path))?,
            cgroup: File::open(format!("{}/cgroup", &ns_base_path))?,
            ipc: File::open(format!("{}/ipc", &ns_base_path))?,
            pid: File::open(format!("{}/pid", &ns_base_path))?,
            user: File::open(format!("{}/user", &ns_base_path))?,
            mnt: File::open(format!("{}/mnt", &ns_base_path))?,
            uts: File::open(format!("{}/uts", &ns_base_path))?,
        })
    }

    fn setns_net(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.net.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }

    fn setns_cgroup(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.cgroup.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }

    fn setns_ipc(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.ipc.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }

    fn setns_pid(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.pid.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }

    fn setns_user(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.user.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }

    fn setns_mnt(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.mnt.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }

    fn setns_uts(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.uts.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }
}

mod tests {
    use super::*;
    use std::fs::{self, File};

    #[test]
    // #[ignore]
    fn test_copy_dir_recursively() {
        fs::create_dir_all("./upper/dir1/dir2");
        File::create("./upper/file1");
        File::create("./upper/file2");
        File::create("./upper/dir1/file3");
        File::create("./upper/dir1/dir2/file3");
        fs::create_dir("./upper_copy");

        match copy_dir_recursively("./upper", "./upper_copy") {
            Ok(num) => {
                fs::remove_dir_all("./upper");
                fs::remove_dir_all("./upper_copy");
            }
            Err(e) => {
                fs::remove_dir_all("./upper");
                fs::remove_dir_all("./upper_copy");
            }
        };
    }
}
