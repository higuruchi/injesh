use crate::command::SubCommand;
use crate::init::Init;
use crate::list::List;
use crate::launch::Launch;
use crate::delete::Delete;
use crate::exec::Exec;

pub struct HandlerStruct<I, L, LA, E, D>
    where I: Init,
        L: List,
        LA: Launch,
        E: Exec,
        D: Delete
{
    command: SubCommand,
    init: I,
    list: L,
    launch: LA,
    exec: E,
    delete: D
}

pub trait Handler {
    fn run(&self);
}

impl<I, L, LA, E, D> Handler for HandlerStruct<I, L, LA, E, D>
    where I: Init,
        L: List,
        LA: Launch,
        E: Exec,
        D: Delete
{
    fn run(&self) {
        println!("hello my name is handler and i have {:?}", self.command);

        match &self.command {
            SubCommand::Init => self.init.init(),
            SubCommand::List => self.list.list(),
            SubCommand::Delete(d) => self.delete.delete(d),
            SubCommand::Exec(e) => self.exec.exec(e),
            SubCommand::File(_) => println!("TODO: file"),
            SubCommand::Launch(l) => self.launch.launch(l)
        }
    }
}

impl<I, L, LA, E, D> HandlerStruct<I, L, LA, E, D>
    where I: Init,
        L: List,
        LA: Launch,
        E: Exec,
        D: Delete
{
    pub fn new(command: SubCommand,
                init: I,
                list: L,
                launch: LA,
                exec: E,
                delete: D
    ) -> impl Handler {
        HandlerStruct {
            command: command,
            init: init,
            list: list,
            launch: launch,
            exec: exec,
            delete: delete
        }
    }
}