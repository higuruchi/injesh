use crate::command;

pub trait Init {
    fn init(&self, init: &command::Init) -> Result<(), Box<dyn std::error::Error>>;
}
