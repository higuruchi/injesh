use crate::command;
use crate::delete::Delete;

pub struct DeleteStruct;

impl Delete for DeleteStruct {
    fn delete(&self, delete: &command::Delete) -> Result<(), Box<dyn std::error::Error>> {
        println!("execute delete!");
        Ok(())
    }
}

impl DeleteStruct {
    pub fn new() -> impl Delete {
        DeleteStruct
    }
}
