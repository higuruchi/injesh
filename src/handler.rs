use crate::command::SubCommand;
use crate::image_downloader::Downloader;
use crate::setting;

use crate::cmd::delete::DeleteStruct;
use crate::cmd::exec::ExecStruct;
use crate::cmd::init::InitStruct;
use crate::cmd::launch::LaunchStruct;
use crate::cmd::list::ListStruct;

pub struct HandlerStruct<DO, RW>
where
    DO: Downloader,
    RW: setting::Reader + setting::Writer,
{
    command: SubCommand<DO, RW>,
    init: InitStruct,
    list: ListStruct,
    launch: LaunchStruct,
    exec: ExecStruct,
    delete: DeleteStruct,
}

pub trait Handler {
    fn run(&mut self);
}

impl<DO, RW> Handler for HandlerStruct<DO, RW>
where
    DO: Downloader,
    RW: setting::Reader + setting::Writer,
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

impl<DO, RW> HandlerStruct<DO, RW>
where
    DO: Downloader,
    RW: setting::Reader + setting::Writer,
{
    pub fn new(
        command: SubCommand<DO, RW>,
        init: InitStruct,
        list: ListStruct,
        launch: LaunchStruct,
        exec: ExecStruct,
        delete: DeleteStruct,
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
