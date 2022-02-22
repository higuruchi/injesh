use crate::command;

pub trait Delete {
    fn delete(&self, delete: &command::Delete) -> Result<(), Box<dyn std::error::Error>>;
}
