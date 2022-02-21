use crate::command;

pub trait Delete {
    fn delete(&self, delete: &command::Delete);
}
