use crate::container;

use nix::{
    mount::{mount, MsFlags},
    unistd::{Gid, Uid},
};
use std::{
    error, fmt,
    fs::{copy, create_dir, read_dir, OpenOptions},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Error {
    InvalidPath(PathBuf),
    OvarlayfsDirInvalid,
    MountFailed(nix::errno::Errno),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidPath(path) => write!(f, "cmd::common::InvalidPath: {:?}", path),
            Error::OvarlayfsDirInvalid => write!(f, "cmd::common::OvarlayfsDirInvalid"),
            Error::MountFailed(why) => write!(f, "cmd::command::MountFailed: because of {}", why),
        }
    }
}

impl error::Error for Error {}

pub fn new_uidmap(uid: &Uid) -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::CString;
    let mut uidmap_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/proc/self/uid_map")?;
    let uidmap = CString::new(format!("0 {} 1", uid.as_raw()))?;

    use std::io::Write;
    uidmap_file.write(uidmap.as_bytes())?;

    Ok(())
}

pub fn new_gidmap(gid: &Gid) -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::CString;
    let mut setgroups_file = OpenOptions::new()
        .write(true)
        .open("/proc/self/setgroups")?;

    use std::io::Write;
    setgroups_file.write(b"deny")?;

    let mut gidmap_file = OpenOptions::new().write(true).open("/proc/self/gid_map")?;
    let gidmap = CString::new(format!("0 {} 1", gid.as_raw()))?;

    gidmap_file.write(gidmap.as_bytes())?;

    Ok(())
}

#[allow(dead_code)]
fn copy_dir_recursively(src: &PathBuf, dest: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let src_pathbuf = src.clone();
    let dest_pathbuf = dest.clone();
    let src_path = src.as_path();
    let dest_path = dest.as_path();

    if !src_path.is_dir() {
        Err(Error::InvalidPath(src_pathbuf))?
    }
    if !dest_path.is_dir() {
        Err(Error::InvalidPath(dest_pathbuf))?
    }

    for entry_result in read_dir(src)? {
        let entry = entry_result?;
        let entry_path = entry.path();
        let dest_entry_path = dest_path.join(
            &entry_path
                .file_name()
                .ok_or(Error::InvalidPath(entry_path.clone()))?,
        );

        if entry.file_type()?.is_dir() {
            create_dir(&dest_path)?;
            copy_dir_recursively(&entry_path, &dest_entry_path)?;
        } else {
            copy(entry_path, &dest_path)?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn overwrite_target_upperdir_by_own_upperdir(
    container_dir_path: &Path,
    overlayfs_dirs: &container::Container,
) -> Result<(), Box<dyn std::error::Error>> {
    let target_upperdir = overlayfs_dirs.upperdir();
    let own_upperdir = container_dir_path.join("upper");

    copy_dir_recursively(&own_upperdir, target_upperdir)?;

    Ok(())
}

#[allow(dead_code)]
fn mount_merged_directory(
    overlayfs_dirs: &container::Container,
) -> Result<(), Box<dyn std::error::Error>> {
    let lowerdir_string = overlayfs_dirs
        .lowerdir()
        .clone()
        .into_os_string()
        .into_string()
        .map_err(|_| Error::OvarlayfsDirInvalid)?;
    let upperdir_string = overlayfs_dirs
        .upperdir()
        .clone()
        .into_os_string()
        .into_string()
        .map_err(|_| Error::OvarlayfsDirInvalid)?;
    let workdir_string = overlayfs_dirs
        .workdir()
        .clone()
        .into_os_string()
        .into_string()
        .map_err(|_| Error::OvarlayfsDirInvalid)?;
    mount(
        Some("overlay"),
        overlayfs_dirs.mergeddir(),
        Some("overlay"),
        MsFlags::empty(),
        Some(
            format!(
                "lowerdir={},upperdir={},workdir={}",
                lowerdir_string, upperdir_string, workdir_string
            )
            .as_str(),
        ),
    )
    .map_err(|why| Error::MountFailed(why))?;

    Ok(())
}
