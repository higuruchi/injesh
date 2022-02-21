use crate::init::Init;
use crate::command::init_error::Error;
use std::{fs, env};

pub struct InitStruct {}

impl Init for InitStruct {
    fn init(&self) -> Result<(), Error> {
        let mut  err_flg = 0;

        let home_injesh = match env::var("HOME") {
            Ok(val) => format!("{}/.injesh", val),
            Err(_) => return Err(Error::HOMENotFound)
        };

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
            return Err(Error::AlreadyInitialized);
        }

        Ok(())
    }
}

impl InitStruct {
    pub fn new() -> impl Init {
        InitStruct{}
    }
}