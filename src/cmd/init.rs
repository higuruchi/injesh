use crate::init_inteface::Init;
use std::{fs, env};

pub struct InitStruct {}

impl Init for InitStruct {
    fn init(&self) {
        let home = match env::var("HOME") {
            Ok(val) => val,
            Err(_) => String::new()
        };

        match fs::create_dir(format!("{}/.injesh", home)) {
            Ok(_) => {},
            Err(why) => println!("{}", why)
        }
        match fs::create_dir(format!("{}/.injesh/images", home)) {
            Ok(_) => {},
            Err(why) => println!("{}", why)
        }
        match fs::create_dir(format!("{}/.injesh/containers", home)) {
            Ok(_) => {},
            Err(why) => println!("{}", why)
        }
        println!("Initialized");
    }
}

impl InitStruct {
    pub fn new() -> impl Init {
        InitStruct{}
    }
}