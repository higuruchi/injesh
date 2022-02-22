use crate::exec::Exec;
use crate::command;

pub struct ExecStruct;

impl Exec for ExecStruct {
    fn exec(&self, exec: &command::Exec)  -> Result<(), Box<dyn std::error::Error>> {
        println!("execute exec!");
        Ok(())
    }
}

impl ExecStruct {
    pub fn new() -> impl Exec {
        ExecStruct
    }
}

