use crate::command;

pub struct ExecStruct;

impl ExecStruct {
    pub fn exec(&self, exec: &command::Exec) -> Result<(), Box<dyn std::error::Error>> {
        println!("execute exec!");
        Ok(())
    }

    pub fn new() -> ExecStruct {
        ExecStruct
    }
}
