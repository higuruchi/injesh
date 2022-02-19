use injesh::parser;
use injesh::init;
use injesh::handler::{self, Handler};

fn main() {
    let command = parser::parse().unwrap();
    let init = init::InitStruct::new();
    let handler = handler::HandlerStruct::new(command, init);


    handler.run();
}