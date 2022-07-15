use crate::command;

use std::fs;
use std::path::Path;

pub struct InitStruct {}

impl InitStruct {
    pub fn new() -> InitStruct {
        InitStruct {}
    }

    pub fn init(&self, init: &command::Init) -> Result<(), Box<dyn std::error::Error>> {
        let user_dirs = init.user();

        if !Path::new(user_dirs.injesh_home()).exists() {
            fs::create_dir(user_dirs.injesh_home())?;
        }
        if !Path::new(user_dirs.images()).exists() {
            fs::create_dir(user_dirs.images())?;
        }
        if !Path::new(user_dirs.containers()).exists() {
            fs::create_dir(user_dirs.containers())?;
        }

        Ok(())
    }
}
