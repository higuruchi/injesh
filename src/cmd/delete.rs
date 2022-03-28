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
        let docker_container_id =
            container::Container::convert_injesh_name_to_docker_id(injesh_container_name)?;
        container::Container::new(&docker_container_id)?.restart()?;

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
    let own_upperdir = container_dir_path.join("upper");

    // copy_dir_recursively(&own_upperdir, target_upperdir).map_err(|why| Error::CopyFailed(why))?;
    copy_dir_recursively(&own_upperdir, target_upperdir)?;

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

fn copy_dir_recursively(
    src: &std::path::PathBuf,
    dest: &std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
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

    for entry_result in fs::read_dir(src)? {
        let entry = entry_result?;
        let entry_path = entry.path();
        let dest_entry_path = dest_path.join(
            &entry_path
                .file_name()
                .ok_or(Error::InvalidPath(entry_path.clone()))?,
        );

        if entry.file_type()?.is_dir() {
            fs::create_dir(&dest_path)?;
            copy_dir_recursively(&entry_path, &dest_entry_path)?;
        } else {
            fs::copy(entry_path, &dest_path)?;
        }
    }
    Ok(())
}
