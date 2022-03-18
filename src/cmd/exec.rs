use crate::command::{self, exec_error::Error};
use crate::container;
use crate::exec::Exec;
use crate::user;

use nix::sched::{setns, CloneFlags};
use nix::unistd::fchdir;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path;
use std::process;

pub struct ExecStruct;

struct FdFiles {
    cgroup: File,
    ipc: File,
    mnt: File,
    net: File,
    pid: File,
    uts: File,
}

impl FdFiles {
    fn new(pid: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let base_path = format!("/proc/{}/ns", pid);
        let base_path = path::Path::new(&base_path);

        if !base_path.exists() {
            Err(Error::DockerProcessNotExists)?
        }

        Ok(FdFiles {
            cgroup: File::open(base_path.join("cgroup"))?,
            ipc: File::open(base_path.join("ipc"))?,
            mnt: File::open(base_path.join("mnt"))?,
            net: File::open(base_path.join("net"))?,
            pid: File::open(base_path.join("pid"))?,
            uts: File::open(base_path.join("uts"))?,
        })
    }
}

impl Exec for ExecStruct {
    fn exec(&self, exec: &command::Exec) -> Result<(), Box<dyn std::error::Error>> {
        // 1. check if container exists
        //     - docker
        // let target_container_info = container::Container::new(docker_name_or_id)?;
        // check_docker_container_exists(docker_name_or_id)?;
        // ** debug **
        let target_pid: u32 = 980525;
        //     - injesh
        // let injesh_container_name = exec.name();
        // check_injesh_container_exists(injesh_container_name)?;

        // 2. get params for setns
        // (got at exec_closure_impl)

        // 3. mk exec closure(will be called as child process)
        let exec_closure = || -> isize {
            exec_closure_impl(target_pid, exec).unwrap_or_else(|err| {
                eprintln!("injesh error: {}", err);
                1
            })
        };

        // 4. clone and run closure
        // clone: allocating tty fail because of fd related error.
        // clone(Box::new(exec_closure), &mut vec![0; 1024 * 1024], CloneFlags::empty(), None)?;
        exec_closure();

        // ----- Dead code -----

        Ok(())
    }
}

fn check_docker_container_exists(
    container_name_or_id: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // TODO: get docker name or id
    // let docker_name_or_id;
    // container::Container::new(docker_name_or_id)?;

    Ok(true)
}

fn check_injesh_container_exists(container_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let user_info = user::User::new()?;
    let injesh_container_path = user_info.containers();
    if path::Path::new(&format!("{}/{}", injesh_container_path, container_name)).exists() {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn exec_closure_impl(
    target_container_pid: u32,
    exec: &command::Exec,
) -> Result<isize, Box<dyn std::error::Error>> {
    // 2. get params for setns
    //     - fd(from pid)
    let fd_files = FdFiles::new(target_container_pid)?;
    //     - mount point(mergeddir)
    // let mergeddir_path = &target_container_info.mergeddir();
    // ** debug **
    let mergeddir_path = &path::Path::new("/var/lib/docker/overlay2/8a07b0e31c72380ed6993dd4c747909cfdc58862c93b0e1d69dd5d8d2899c2d9/merged").to_path_buf();

    // save mergeddir path before changing root by setns
    let mergeddir = File::open(mergeddir_path)?;

    // 3. mk exec closure(will be called as child process)
    //     1. setns
    setns(fd_files.cgroup.as_raw_fd(), CloneFlags::CLONE_NEWCGROUP)?;
    setns(fd_files.ipc.as_raw_fd(), CloneFlags::CLONE_NEWIPC)?;
    setns(fd_files.uts.as_raw_fd(), CloneFlags::CLONE_NEWUTS)?;
    setns(fd_files.net.as_raw_fd(), CloneFlags::CLONE_NEWNET)?;
    setns(fd_files.pid.as_raw_fd(), CloneFlags::CLONE_NEWPID)?;
    setns(fd_files.mnt.as_raw_fd(), CloneFlags::CLONE_NEWNS)?;

    //     2. chroot, chdir
    // chroot: path not found
    // chroot(mergeddir_path)?;
    fchdir(mergeddir.as_raw_fd())?;

    //     3. exec cmd
    let cmd_and_args: (&str, Option<Vec<&str>>) = split_cmd(exec.cmd())?;
    process::Command::new(cmd_and_args.0)
        .args(cmd_and_args.1.unwrap_or(Vec::new()))
        .exec();

    // ----- Dead code -----
    // Detail: https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html#tymethod.exec

    // exit code
    Ok(0)
}

// If cmd is None, return ("/bin/bash", None)
// If cmd has some string, split it to primary command and args
// e.g.
// "echo hello world" -> ("echo", ["hello", "world"])
// "date" -> ("date", None)
fn split_cmd(cmd: Vec<&str>) -> Result<(&str, Option<Vec<&str>>), Box<dyn std::error::Error>> {
    const DEFAULT_CMD: &str = "/bin/bash";
    // If cmd is not provided, return (DEFAULT_CMD, None)
    if cmd.is_empty() {
        Ok((DEFAULT_CMD, None))
    } else {
        // If cmd has string,
        // attempt to split it to primary command and args as str type
        // and if white space isn't found, None is bound to.
        // e.g.
        // "echo hello world" -> Some("echo", "hello world")
        // "date" -> None
        let primary_cmd: &str = cmd[0];
        let args: Option<Vec<&str>> = if cmd.len() > 1 {
            Some(cmd[1..].to_vec())
        } else {
            None
        };
        Ok((primary_cmd, args))
    }
}

impl ExecStruct {
    pub fn new() -> impl Exec {
        ExecStruct
    }
}

// ref: strace of `nsenter --all`:
// openat(AT_FDCWD, "/proc/980525/ns/cgroup", O_RDONLY) = 5
// openat(AT_FDCWD, "/proc/980525/ns/ipc", O_RDONLY) = 6
// openat(AT_FDCWD, "/proc/980525/ns/uts", O_RDONLY) = 7
// openat(AT_FDCWD, "/proc/980525/ns/net", O_RDONLY) = 8
// openat(AT_FDCWD, "/proc/980525/ns/pid", O_RDONLY) = 9
// openat(AT_FDCWD, "/proc/980525/ns/mnt", O_RDONLY) = 10
// setns(5, CLONE_NEWCGROUP)               = 0
// close(5)                                = 0
// setns(6, CLONE_NEWIPC)                  = 0
// close(6)                                = 0
// setns(7, CLONE_NEWUTS)                  = 0
// close(7)                                = 0
// setns(8, CLONE_NEWNET)                  = 0
// close(8)                                = 0
// setns(9, CLONE_NEWPID)                  = 0
// close(9)                                = 0
// setns(10, CLONE_NEWNS)                  = 0
// close(10)                               = 0
// fchdir(3)                               = 0
// chroot(".")                             = 0
// close(3)                                = 0
// fchdir(4)                               = 0
// close(4)                                = 0
// clone(child_stack=NULL, flags=CLONE_CHILD_CLEARTID|CLONE_CHILD_SETTID|SIGCHLD, child_tidptr=0x7fc5e7c3a6d0) = 981560
// wait4(981560, [{WIFEXITED(s) && WEXITSTATUS(s) == 0}], WSTOPPED, NULL) = 981560
