use crate::init::Init;
use crate::command::Error;
use std::{fs, env};

pub struct InitStruct {}

impl Init for InitStruct {
    fn init(&self) -> Result<(), Error> {
        let mut  err_flg = 0;

        let home = match env::var("HOME") {
            Ok(val) => val,
            Err(_) => return Err(Error::HOMENouFound)
        };

        match fs::create_dir(format!("{}/.injesh", home)) {
            Ok(_) => {},
            Err(_) => err_flg += 1
        }
        match fs::create_dir(format!("{}/.injesh/images", home)) {
            Ok(_) => {},
            Err(_) => err_flg += 1
        }
        match fs::create_dir(format!("{}/.injesh/containers", home)) {
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