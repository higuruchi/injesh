use injesh::parser;
use injesh::cmd::{init, list, launch, delete, exec};
use injesh::handler::{self, Handler};

fn main() {
    let command = parser::parse().unwrap();
    let init = init::InitStruct::new();
    let list = list::ListStruct::new();
    let launch = launch::LaunchStruct::new();
    let delete = delete::DeleteStruct::new();
    let exec = exec::ExecStruct::new();
    let handler = handler::HandlerStruct::new(command, init, list, launch, exec, delete);


    handler.run();
}