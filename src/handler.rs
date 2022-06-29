use crate::command::SubCommand;
use crate::delete::Delete;
use crate::exec::Exec;
use crate::image_downloader::Downloader;
use crate::init::Init;
use crate::launch::Launch;
use crate::list::List;
use crate::setting;

pub struct HandlerStruct<I, L, LA, E, D, DO>
where
    I: Init,
    L: List,
    LA: Launch<DO>,
    E: Exec,
    D: Delete,
    DO: Downloader,
{
    command: SubCommand<DO>,
    init: I,
    list: L,
    launch: LA,
    exec: E,
    delete: D,
}

pub trait Handler {
    fn run(&mut self);
}

impl< I, L, LA, E, D, DO> Handler for HandlerStruct<I, L, LA, E, D, DO>
where
    I: Init,
    L: List,
    LA: Launch<DO>,
    E: Exec,
    D: Delete,
    DO: Downloader,
{
    fn run(&mut self) {
        match &mut self.command {
            // TODO: エラーハンドリング
            SubCommand::Init(init_args) => match self.init.init(init_args) {
                Ok(_) => println!("Initialized!"),
                Err(e) => println!("Initialize Error {:?}", e),
            },
            SubCommand::List(list_args) => match self.list.list(list_args) {
                Ok(_) => {}
                Err(e) => println!("execute list command error: {}", e),
            },
            SubCommand::Delete(delete_args) => match self.delete.delete(delete_args) {
                Ok(_) => {}
                Err(e) => println!("execute delete command error: {}", e),
            },
            SubCommand::Exec(exec_args) => match self.exec.exec(exec_args) {
                Ok(_) => {}
                Err(e) => println!("execute exec command error: {:?}", e),
            },
            SubCommand::File(_) => println!("TODO: file sub command"),
            SubCommand::Launch(launch_args) => match self.launch.launch(launch_args) {
                Ok(_) => {}
                Err(e) => println!("execute launch command error: {:?}", e),
            },
        }
    }
}

impl<I, L, LA, E, D, DO> HandlerStruct<I, L, LA, E, D, DO>
where
    I: Init,
    L: List,
    LA: Launch<DO>,
    E: Exec,
    D: Delete,
    DO: Downloader,
{
    pub fn new(
        command: SubCommand<DO>,
        init: I,
        list: L,
        launch: LA,
        exec: E,
        delete: D,
    ) -> impl Handler {
        HandlerStruct {
            command: command,
            init: init,
            list: list,
            launch: launch,
            exec: exec,
            delete: delete,
        }
    }
}
