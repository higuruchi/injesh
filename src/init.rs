use crate::command::Error;

pub trait Init {
    fn init(&self) -> Result<(), Error>;
}