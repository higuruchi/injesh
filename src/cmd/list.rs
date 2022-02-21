use crate::list::List;
use std::{env, fs};

pub struct ListStruct;

impl List for ListStruct {
    fn list(&self) {
        // println!("execute list!");
        let homedir = match env::var("HOME") {
            Ok(path) => path,
            Err(_) => String::new(),
        };
        let containers_root = format!("{HOME}/.injesh/containers", HOME = homedir);
        // debug
        // let containers_root = format!("{}/target", env::var("PWD").unwrap());
        let mut container_names = String::new();

        match fs::read_dir(containers_root) {
            Ok(paths) => {
                for path in paths.flatten() {
                    if path.path().is_dir() {
                        let container_name = match path.file_name().into_string() {
                            Ok(name) => name,
                            Err(_) => String::from("<none>"),
                        };
                        container_names += &format!("{}\n", container_name);
                    }
                }
            }
            Err(why) => eprintln!("{:?}", why.kind()),
        }

        println!("{}", container_names.trim());
    }
}

impl ListStruct {
    pub fn new() -> impl List {
        ListStruct
    }
}
