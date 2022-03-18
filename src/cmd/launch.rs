use crate::command::{self, RootFSOption};
use crate::image_downloader::Downloader;
use crate::launch::Launch;
use crate::utils;
use std::ffi::CString;
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
    UpperDirNotFound,
    LowerDirNotFound,
    WorkDirNotFound,
    InvalidRootFSPath,
    NotImplemented,
    Fork,
    Waitpid,
    UnmountFailed(nix::errno::Errno),
    MountFailed(nix::errno::Errno),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AlreadyExists => write!(f, "Debug Container Already Exists"),
            Umount => write!(f, "umount error"),
            UpperDirNotFound => write!(f, "upper dir not found"),
            LowerDirNotFound => write!(f, "lower dir not found"),
            WorkDirNotFound => write!(f, "workdir not found"),
            InvalidRootFSPath => write!(f, "invalid rootfs path"),
            NotImplemented => write!(f, "Not implemented"),
            Fork => write!(f, "failed fork"),
            Waitpid => write!(f, "failed waitpid"),
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
    /// /var/lib/docker/overlay2/<CONTAINER_ID>/upperを~/.injesh/containers/<hoge>/upperに対してコピーする
    /// デバック対象コンテナのlowerdirに対してrootfsを追加した後reloadする
    /// デバック対象コンテナのプロセスIDを取得
    /// forkする
    /// 取得したデバック対象コンテナプロセスIDをもとにsetnsをし、名前空間を同一にする
    /// 与えられた初期実行ファイルをexecする
    fn launch(&self, launch: &command::Launch<DO>) -> Result<(), Box<dyn std::error::Error>> {
        // injeshコマンドが初期化されてるかどうかチェック
        utils::check_initialized()?;

        // デバックコンテナの設定ファイル、ディレクトリ周りの初期化
        initialize_setting(launch)?;

        // overlyafsのマウントし直し
        remount(launch)?;

        // TODO: デバック対象コンテナのlowerdirに対してrootfsを追加した後reloadする

        // デバック対象コンテナのプロセスIDとネームスペースのファイルディスクリプタを取得
        let container_pid = launch.target_container().pid();
        let ns = Ns::new(container_pid)?;

        unsafe {
            match fork() {
                // 親プロセスの場合
                Ok(ForkResult::Parent { child, .. }) => match waitpid(child, None) {
                    Ok(status) => println!("Child exited {:?}", status),
                    Err(_) => return Err(Error::Waitpid)?,
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

                    let vec: Vec<CString> = Vec::new();
                    // execでプログラムを実行
                    let cmd = match launch.cmd() {
                        Some(cmd) => cmd,
                        None => "/bin/bash",
                    };
                    println!("cmd{:?}", cmd);
                    execv(&CString::new(cmd)?, &vec)?;
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

fn initialize_setting<DO: Downloader>(
    launch: &command::Launch<DO>,
) -> Result<(), Box<dyn std::error::Error>> {
    // rootfsの種類などが記載された設定ファイルsetting.yamlを~/.injesh/containers/に作成する
    match Path::new(&format!("{}/{}", launch.user().containers(), launch.name())).exists() {
        true => return Err(Error::AlreadyExists)?,
        false => {
            fs::create_dir_all(format!(
                "{}/{}/upper",
                launch.user().containers(),
                launch.name()
            ))?;
            let mut setting_file = fs::File::create(format!(
                "{}/{}/setting.yaml",
                launch.user().containers(),
                launch.name()
            ))?;
            // TODO: 設定ファイルの形式を決めてない
            setting_file.write_all(b"content of setting.yaml")?;
        }
    }

    // /var/lib/docker/overlay2/<CONTAINER_ID>/upperを~/.injesh/containers/<hoge>/upperに対してコピーする
    for entry in fs::read_dir(launch.target_container().upperdir())? {
        let dir = entry?;
        let path = dir.path();
        let file_name = match path.file_name() {
            Some(file_name_os_str) => match file_name_os_str.to_str() {
                Some(file_name_str) => file_name_str,
                None => continue,
            },
            None => continue,
        };

        fs::copy(
            &path,
            format!(
                "{}/{}/upper/{}",
                launch.user().containers(),
                launch.name(),
                file_name
            ),
        )?;
    }

    Ok(())
}

fn remount<DO: Downloader>(launch: &command::Launch<DO>) -> Result<(), Box<dyn std::error::Error>> {
    let rootfs_path = match launch.rootfs_option() {
        RootFSOption::RootfsImage(image) => {
            match image.check_rootfs_newest() {
                Ok(flg) => match flg {
                    true => { /* do nothing */ }
                    false => {
                        image.download_image()?;
                    }
                },
                Err(e) => {
                    return Err(e)?;
                }
            }
            image.rootfs_path()
        }
        _ => return Err(Error::NotImplemented)?,
    };
    let lower_dir = launch
        .target_container()
        .lowerdir()
        .to_str()
        .ok_or(Error::LowerDirNotFound)?;
    let upper_dir = launch
        .target_container()
        .upperdir()
        .to_str()
        .ok_or(Error::UpperDirNotFound)?;
    let work_dir = launch
        .target_container()
        .workdir()
        .to_str()
        .ok_or(Error::WorkDirNotFound)?;

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
        Ok(Ns {
            net: File::open(format!("/proc/{}/ns/net", container_pid))?,
            cgroup: File::open(format!("/proc/{}/ns/cgroup", container_pid))?,
            ipc: File::open(format!("/proc/{}/ns/ipc", container_pid))?,
            pid: File::open(format!("/proc/{}/ns/pid", container_pid))?,
            user: File::open(format!("/proc/{}/ns/user", container_pid))?,
            mnt: File::open(format!("/proc/{}/ns/mnt", container_pid))?,
            uts: File::open(format!("/proc/{}/ns/uts", container_pid))?,
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
