use crate::command;
use crate::launch::Launch;

pub struct LaunchStruct;

impl Launch for LaunchStruct {
    fn launch(&self, launch: &command::Launch) -> Result<(), Box<dyn std::error::Error>> {
        println!("execute launch!");
        Ok(())
    }
}

impl LaunchStruct {
    pub fn new() -> impl Launch {
        LaunchStruct
    }
}
