use crate::command;

pub trait Launch {
    fn launch(&self, launch: &command::Launch);
}