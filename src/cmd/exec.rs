use crate::{cmd::common, command, container, image_downloader, namespace, setting, user, utils};

use nix::sched::{unshare, CloneFlags};
use nix::sys::wait::waitpid;
use nix::unistd::{chdir, chroot, fork, ForkResult, Gid, Uid};
use std::path::PathBuf;
use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    Waitpid,
    Fork,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Waitpid => write!(f, "cmd::exec: failed waitpid"),
            Error::Fork => write!(f, "cmd::exec: failed fork"),
        }
    }
}

impl error::Error for Error {}

pub struct ExecStruct;

impl ExecStruct {
    pub fn exec<D: image_downloader::Downloader, RW: setting::Reader + setting::Writer>(
        &self,
        exec: &mut command::Exec<D, RW>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        utils::check_initialized()?;

        let dcontainer_setting = exec.setting_mut().read()?;
        let container = container::Container::new(dcontainer_setting.docker_container_id())?;

        let container_pid = container.pid();
        let ns = namespace::Ns::new(container_pid)?;

        unsafe {
            match fork() {
                Ok(ForkResult::Parent { child, .. }) => match waitpid(child, None) {
                    Ok(status) => println!("Child {:?}", status),
                    Err(_) => Err(Error::Waitpid)?,
                },
                Ok(ForkResult::Child) => {
                    let gid = Gid::current();
                    let uid = Uid::current();

                    // setnsで名前空間を変更
                    ns.setns_net()?;
                    ns.setns_cgroup()?;
                    ns.setns_ipc()?;
                    ns.setns_pid()?;
                    ns.setns_uts()?;
                    unshare(CloneFlags::CLONE_NEWUSER)?;

                    common::new_uidmap(&uid)?;
                    common::new_gidmap(&gid)?;

                    let user = user::User::new()?;
                    let dcontainer_base = format!("{}/{}", user.containers(), exec.name());
                    let dcontainer_base_merged = format!("{}/merged", &dcontainer_base);
                    chroot(&PathBuf::from(&dcontainer_base_merged))?;
                    chdir("/")?;

                    // execでプログラムを実行
                    use std::os::unix::process::CommandExt;
                    std::process::Command::new(exec.cmd().main())
                        .args(exec.cmd().detail())
                        .exec();
                }
                Err(_) => return Err(Error::Fork)?,
            }
        };

        Ok(())
    }

    pub fn new() -> ExecStruct {
        ExecStruct
    }
}
