use crate::init_inteface::Init;

pub struct InitStruct {}

impl Init for InitStruct {
    fn init(&self) {
        println!("execute init");
        return;
    }
}

impl InitStruct {
    pub fn new() -> impl Init {
        InitStruct{}
    }
}