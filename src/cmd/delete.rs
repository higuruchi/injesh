use crate::{
    command::{self, delete_error::Error},
    user, utils,
};

use nix::mount::{umount2, MntFlags};
use std::{fs, path};

pub struct DeleteStruct;

impl DeleteStruct {
    pub fn delete(&self, delete: &command::Delete) -> Result<(), Box<dyn std::error::Error>> {
        utils::check_initialized()?;
        let injesh_container_name = delete.name();

        // ~/.injesh/containers/<injesh_container_name>
        let container_dir_path = format!(
            "{}/{}",
            user::User::new()?.containers(),
            injesh_container_name
        );
        let container_merged_dir_path = format!("{}/merged", container_dir_path);
        let container_dir_path = path::Path::new(&container_dir_path);
        let container_merged_dir_path = path::Path::new(&container_merged_dir_path);

        check_container_exists(container_dir_path)?;

        umount2(container_merged_dir_path, MntFlags::empty())
            .map_err(|why| Error::UnmountFailed(why))?;

        fs::remove_dir_all(container_dir_path).map_err(|why| Error::RemoveFailed(why))?;

        Ok(())
    }

    pub fn new() -> DeleteStruct {
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
