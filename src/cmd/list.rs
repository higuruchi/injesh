use crate::list_interface::List;

pub struct ListStruct;

impl List for ListStruct {
    fn list(&self) {}
}

impl ListStruct {
    pub fn new() -> impl List {
        ListStruct
    }
}