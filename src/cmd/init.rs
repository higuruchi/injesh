use crate::init::Init;
use crate::command::init_error::Error;
use crate::command;
use std::fs;

pub struct InitStruct {}

impl Init for InitStruct {
    fn init(&self, init: &command::Init) -> Result<(), Box<dyn std::error::Error>> {
        let mut  err_flg = 0;

        let user_dirs = init.user();

        match fs::create_dir(user_dirs.injesh_home()) {
            Ok(_) => {},
            Err(_) => err_flg += 1
        }
        match fs::create_dir(user_dirs.images()) {
            Ok(_) => {},
            Err(_) => err_flg += 1
        }
        match fs::create_dir(user_dirs.containers()) {
            Ok(_) => {},
            Err(_) => err_flg += 1
        }

        if err_flg == 3 {
            Err(Error::AlreadyInitialized)?
        }

        Ok(())
    }
}

impl InitStruct {
    pub fn new() -> impl Init {
        InitStruct{}
    }
}
