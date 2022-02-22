use crate::command;

pub trait Launch {
    fn launch(&self, launch: &command::Launch) -> Result<(), Box<dyn std::error::Error>>;
}