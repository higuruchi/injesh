use crate::command::{self, SubCommand, Exec, Launch, RootFSOption, FileSubCommand, File, Error};
use crate::init_inteface::Init;

pub struct HandlerStruct<T: Init> {
    command: SubCommand,
    init: T
}

pub trait Handler {
    fn run(&self);
}

impl<T: Init> Handler for HandlerStruct<T> {
    fn run(&self) {
        println!("hello my name is handler and i have {:?}", self.command);

        match self.command {
            SubCommand::Init => self.init.init(),
            SubCommand::List => println!("TODO: list"),
            SubCommand::Delete(_) => println!("TODO: delete"),
            SubCommand::Exec(_) => println!("TODO: exec"),
            SubCommand::File(_) => println!("TODO: file"),
            SubCommand::Launch(_) => println!("TODO: launch")
        }
    }
}

impl<T: Init> HandlerStruct<T> {
    pub fn new(command: SubCommand, init: T) -> impl Handler {
        HandlerStruct {
            command: command,
            init: init
        }
    }
}