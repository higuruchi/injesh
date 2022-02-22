use crate::list::List;

pub struct ListStruct;

impl List for ListStruct {
    fn list(&self)  -> Result<(), Box<dyn std::error::Error>> {
        println!("execute list!");
        Ok(())
    }
}

impl ListStruct {
    pub fn new() -> impl List {
        ListStruct
    }
}