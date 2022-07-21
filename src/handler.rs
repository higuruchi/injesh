use crate::cmd::delete::DeleteStruct;
use crate::cmd::exec::ExecStruct;
use crate::cmd::init::InitStruct;
use crate::cmd::launch::LaunchStruct;
use crate::cmd::list::ListStruct;
use crate::command::SubCommand;
use crate::image_downloader::Downloader;
use crate::setting;

pub struct HandlerStruct<DO, RW>
where
    DO: Downloader,
    RW: setting::Reader + setting::Writer,
{
    command: SubCommand<DO, RW>,
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
            SubCommand::Init(init_args) => {
                let init = InitStruct::new();
                match init.init(init_args) {
                    Ok(_) => println!("Initialized!"),
                    Err(e) => println!("Initialize Error {:?}", e),
                }
            }
            SubCommand::List(list_args) => {
                let list = ListStruct::new();
                match list.list(list_args) {
                    Ok(_) => {}
                    Err(e) => println!("execute list command error: {}", e),
                }
            }
            SubCommand::Delete(delete_args) => {
                let delete = DeleteStruct::new();
                match delete.delete(delete_args) {
                    Ok(_) => {}
                    Err(e) => println!("execute delete command error: {}", e),
                }
            }
            SubCommand::Exec(exec_args) => {
                let exec = ExecStruct::new();
                match exec.exec(exec_args) {
                    Ok(_) => {}
                    Err(e) => println!("execute exec command error: {:?}", e),
                }
            }
            SubCommand::File(_) => println!("TODO: file sub command"),
            SubCommand::Launch(launch_args) => {
                let launch = LaunchStruct::new();
                match launch.launch(launch_args) {
                    Ok(_) => {}
                    Err(e) => println!("execute launch command error: {:?}", e),
                }
            }
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
    ) -> impl Handler {
        HandlerStruct {
            command: command,
        }
    }
}
