use crate::command;

pub trait Delete {
    fn delete(&self, delete: &str);
}