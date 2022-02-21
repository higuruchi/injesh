use crate::delete::Delete;
use crate::command;

pub struct DeleteStruct;

impl Delete for DeleteStruct {
    fn delete(&self, delete: &command::Delete) {
        println!("execute delete!")
    }
}

impl DeleteStruct {
    pub fn new() -> impl Delete {
        DeleteStruct
    }
}
