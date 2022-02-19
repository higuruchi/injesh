use crate::command;

pub trait Exec {
    fn exec(&self, exec: &command::Exec);
}