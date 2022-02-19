use crate::exec::Exec;
use crate::command;

pub struct ExecStruct;

impl Exec for ExecStruct {
    fn exec(&self, exec: &command::Exec) {
        println!("execute exec!");
    }
}

impl ExecStruct {
    pub fn new() -> impl Exec {
        ExecStruct
    }
}

