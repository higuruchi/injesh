use crate::command;
use crate::exec::Exec;

pub struct ExecStruct;

impl Exec for ExecStruct {
    fn exec(&self, exec: &command::Exec) -> Result<(), Box<dyn std::error::Error>> {
        println!("execute exec!");
        Ok(())
    }
}

impl ExecStruct {
    pub fn new() -> impl Exec {
        ExecStruct
    }
}
