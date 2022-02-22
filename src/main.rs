use injesh::cmd::{delete, exec, init, launch, list};
use injesh::handler::{self, Handler};
use injesh::parser;

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
