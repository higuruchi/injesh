use crate::init::Init;
use crate::command::init_error::Error;
use crate::command;
use std::fs;

pub struct InitStruct {}

impl Init for InitStruct {
    fn init(&self, init: &command::Init) -> Result<(), Box<dyn std::error::Error>> {
        let mut  err_flg = 0;

        let home_injesh = format!("{}/.injesh", init.user().home());

        match fs::create_dir(format!("{}", home_injesh)) {
            Ok(_) => {},
            Err(_) => err_flg += 1
        }
        match fs::create_dir(format!("{}/images", home_injesh)) {
            Ok(_) => {},
            Err(_) => err_flg += 1
        }
        match fs::create_dir(format!("{}/containers", home_injesh)) {
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