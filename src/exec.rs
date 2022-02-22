use crate::command;

pub trait Exec {
    fn exec(&self, exec: &command::Exec) -> Result<(), Box<dyn std::error::Error>>;
}
