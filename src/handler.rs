use crate::command::{self, SubCommand, Exec, Launch, RootFSOption, FileSubCommand, File, Error};
use crate::init_inteface::Init;
use crate::list_interface::List;

pub struct HandlerStruct<I, L>
    where I: Init,
        L: List
{
    command: SubCommand,
    init: I,
    list: L
}

pub trait Handler {
    fn run(&self);
}

impl<I, L> Handler for HandlerStruct<I, L>
    where I: Init,
        L: List
{
    fn run(&self) {
        println!("hello my name is handler and i have {:?}", self.command);

        match self.command {
            SubCommand::Init => self.init.init(),
            SubCommand::List => self.list.list(),
            SubCommand::Delete(_) => println!("TODO: delete"),
            SubCommand::Exec(_) => println!("TODO: exec"),
            SubCommand::File(_) => println!("TODO: file"),
            SubCommand::Launch(_) => println!("TODO: launch")
        }
    }
}

impl<I, L> HandlerStruct<I, L>
    where I: Init,
        L: List
{
    pub fn new(command: SubCommand, init: I, list: L) -> impl Handler {
        HandlerStruct {
            command: command,
            init: init,
            list: list
        }
    }
}