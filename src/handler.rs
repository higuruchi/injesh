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

        match &self.command {
            // TODO: エラーハンドリング
            SubCommand::Init(init) => {
                match self.init.init(init) {
                    Ok(_) => println!("Initialized!"),
                    Err(e) => println!("Initialize Error {:?}", e)
                }
            },
            SubCommand::List => match self.list.list() {
                Ok(_) => {},
                Err(e) => println!("execute list command error: {:?}", e)
            },
            SubCommand::Delete(d) => match self.delete.delete(d) {
                Ok(_) => {},
                Err(e) => println!("execute delete command error: {:?}", e)
            },
            SubCommand::Exec(e) => match self.exec.exec(e) {
                Ok(_) => {},
                Err(e) => println!("execute exec command error: {:?}", e)
            },
            SubCommand::File(_) => println!("TODO: file sub command"),
            SubCommand::Launch(l) => match self.launch.launch(l) {
                Ok(_) => {},
                Err(e) => println!("execute launch command error: {:?}", e)
            }
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