use crate::launch::Launch;
use crate::command;

pub struct LaunchStruct;

impl Launch for LaunchStruct {
    fn launch(&self, launch: &command::Launch) {
        println!("execute launch!");
    }
}

impl LaunchStruct {
    pub fn new() -> impl Launch {
        LaunchStruct
    }
}