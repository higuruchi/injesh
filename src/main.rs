use injesh::parser;
use injesh::cmd::{init, list};
use injesh::handler::{self, Handler};

fn main() {
    let command = parser::parse().unwrap();
    let init = init::InitStruct::new();
    let list = list::ListStruct::new();
    let handler = handler::HandlerStruct::new(command, init, list);


    handler.run();
}