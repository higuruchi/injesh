use crate::command::{self, delete_error::Error};
use crate::container;
use crate::delete::Delete;
use crate::user;
use crate::utils;

use nix::mount::{mount, umount2, MntFlags, MsFlags};
use std::{fs, path};

pub struct DeleteStruct;

impl Delete for DeleteStruct {
    fn delete(&self, delete: &command::Delete) -> Result<(), Box<dyn std::error::Error>> {
        utils::check_initialized()?;
        let injesh_container_name = delete.name();

        // ~/.injesh/containers/<injesh_container_name>
        let container_dir_path = format!(
            "{}/{}",
            user::User::new()?.containers(),
            injesh_container_name
        );
        let container_dir_path = path::Path::new(&container_dir_path);

        // check container is exists
        check_container_exists(container_dir_path)?;
        let overlayfs_dirs = container::Container::new(injesh_container_name)?;

        // umount merged directory
        // as syscall: umount2("/path/to/dest", 0)
        umount2(overlayfs_dirs.mergeddir(), MntFlags::empty())
            .map_err(|why| Error::UnmountFailed(why))?;

        // restore original upperdir
        overwrite_target_upperdir_by_own_upperdir(container_dir_path, &overlayfs_dirs)?;

        // mount merged directory
        mount_merged_directory(&overlayfs_dirs)?;

        // restart container
        container::Container::restart(injesh_container_name)?;

        // delete own containers directory
        fs::remove_dir_all(container_dir_path).map_err(|why| Error::RemoveFailed(why))?;

        Ok(())
    }
}

impl DeleteStruct {
    pub fn new() -> impl Delete {
        DeleteStruct
    }
}

fn check_container_exists(
    container_dir_path: &path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !container_dir_path.exists() {
        Err(Error::ContainerNotFound)?
    }

    Ok(())
}

fn overwrite_target_upperdir_by_own_upperdir(
    container_dir_path: &path::Path,
    overlayfs_dirs: &container::Container,
) -> Result<(), Box<dyn std::error::Error>> {
    let target_upperdir = overlayfs_dirs.upperdir();
    let own_upperdir = container_dir_path.join("upperdir");

    fs::copy(own_upperdir, target_upperdir).map_err(|why| Error::CopyFailed(why))?;

    Ok(())
}

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
