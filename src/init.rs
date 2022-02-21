use crate::command::init_error::Error;

pub trait Init {
    fn init(&self) -> Result<(), Error>;
}