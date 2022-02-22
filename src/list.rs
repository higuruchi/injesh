use crate::command;

pub trait List {
    fn list(&self, list: &command::List) -> Result<(), Box<dyn std::error::Error>>;
}
