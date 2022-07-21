use clap::Parser;
use injesh::command;
use injesh::handler::{self, Handler};
use injesh::image_downloader_lxd;
use injesh::parser;
use injesh::setting_yaml;

fn main() {
    let args: parser::Cli = parser::Cli::parse();
    match args.action {
        parser::Action::Init => {
            let init_command = command::SubCommand::Init::<
                image_downloader_lxd::Downloader,
                setting_yaml::YamlReaderWriter,
            >(parser::initialize_init().unwrap());
            let mut handler = handler::HandlerStruct::new(init_command);
            handler.run();
        }
        parser::Action::Launch(launch) => {
            let launch_command =
                command::SubCommand::Launch(parser::initialize_launch(launch).unwrap());
            let mut handler = handler::HandlerStruct::new(launch_command);
            handler.run();
        }
        parser::Action::Exec(exec) => {
            let exec_command = command::SubCommand::Exec::<
                image_downloader_lxd::Downloader,
                setting_yaml::YamlReaderWriter,
            >(parser::initialize_exec(exec).unwrap());
            let mut handler = handler::HandlerStruct::new(exec_command);
            handler.run();
        }
        parser::Action::List => {
            let list_command = command::SubCommand::List::<
                image_downloader_lxd::Downloader,
                setting_yaml::YamlReaderWriter,
            >(parser::initialize_list().unwrap());
            let mut handler = handler::HandlerStruct::new(list_command);
            handler.run();
        }
        parser::Action::Delete(delete) => {
            let delete_command = command::SubCommand::Delete::<
                image_downloader_lxd::Downloader,
                setting_yaml::YamlReaderWriter,
            >(parser::initialize_delete(delete).unwrap());
            let mut handler = handler::HandlerStruct::new(delete_command);
            handler.run();
        }
        parser::Action::File(file) => match file.action {
            parser::FileAction::Pull(pull) => {
                let file_pull_command = command::SubCommand::File::<
                    image_downloader_lxd::Downloader,
                    setting_yaml::YamlReaderWriter,
                >(command::FileSubCommand::Pull(
                    parser::initialize_file_pull(pull).unwrap(),
                ));
                let mut handler = handler::HandlerStruct::new(file_pull_command);
                handler.run();
            }
            parser::FileAction::Push(push) => {
                let file_push_command = command::SubCommand::File::<
                    image_downloader_lxd::Downloader,
                    setting_yaml::YamlReaderWriter,
                >(command::FileSubCommand::Push(
                    parser::initialize_file_push(push).unwrap(),
                ));
                let mut handler = handler::HandlerStruct::new(file_push_command);
                handler.run();
            }
        },
    };
}
