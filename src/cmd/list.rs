use crate::list::List;

pub struct ListStruct;

impl List for ListStruct {
    fn list(&self) {
        println!("execute list!");
    }
}

impl ListStruct {
    pub fn new() -> impl List {
        ListStruct
    }
}