use nix::sched::{setns, CloneFlags};
use std::fs::File;
use std::os::unix::io::AsRawFd;

/// プロセスのnamespaceファイルディスクリプタを管理する構造体
pub struct Ns {
    net: File,
    cgroup: File,
    ipc: File,
    pid: File,
    user: File,
    mnt: File,
    uts: File,
}

impl Ns {
    pub fn new(container_pid: u32) -> Result<Ns, Box<dyn std::error::Error>> {
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

    pub fn setns_net(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.net.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }

    pub fn setns_cgroup(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.cgroup.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }

    pub fn setns_ipc(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.ipc.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }

    pub fn setns_pid(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.pid.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }

    pub fn setns_user(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.user.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }

    pub fn setns_mnt(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.mnt.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }

    pub fn setns_uts(&self) -> Result<(), Box<dyn std::error::Error>> {
        setns(self.uts.as_raw_fd(), CloneFlags::empty())?;
        Ok(())
    }
}
