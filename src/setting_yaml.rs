use crate::setting::{Reader, Setting, Shell, Writer};
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::{SerializeStruct};
use std::str;

use std::path::PathBuf;
use std::{error, fmt};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;

#[derive(Debug)]
pub enum Error {
    UnExpectedShell,
    UnExpectedCommand,
    UnexpectedContainerId,
    Parse,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnExpectedShell => write!(f, "unexpected shell"),
            UnExpectedCommand => write!(f, "unexpected command"),
            UnexpectedContainerId => write!(f, "unexpected container id"),
            Parse => write!(f, "parse error"),
        }
    }
}

impl error::Error for Error {}

#[derive(Deserialize)]
struct YamlSetting {
    docker_container_id: String,
    shell: String,
    commands: Vec<String>,
}

struct YamlReaderWriter {
    setting_file_path: PathBuf,
}

impl Reader for YamlReaderWriter {
    fn read(&self) -> Result<Setting, Box<dyn std::error::Error>> {
        let setting_file = File::open(&self.setting_file_path)?;
        let setting_yaml: YamlSetting = serde_yaml::from_reader(&setting_file)?;
        let shell = match setting_yaml.shell.as_str() {
            "bash" => Shell::Bash,
            "/bin/bash" => Shell::Bash,
            "sh" => Shell::Sh,
            "/bin/sh" => Shell::Sh,
            _ => return Err(Error::UnExpectedShell)?,
        };

        Ok(Setting::new(
            &setting_yaml.docker_container_id,
            shell,
            setting_yaml.commands,
        ))
    }
}


impl Writer for YamlReaderWriter {
    fn write(&self, setting: &Setting) -> Result<(), Box<dyn std::error::Error>> {
        let commands: Vec<String> = setting.commands().iter().map(|command| {
            command.to_string()
        }).collect();
        let yaml_setting = YamlSetting {
            docker_container_id: setting.docker_container_id().to_string(),
            shell: setting.shell().to_string(),
            commands: commands
        };

        let yaml_string = serde_yaml::to_string(&yaml_setting)?;

        let mut setting_file = OpenOptions::new().write(true).create(true).read(false).open(&self.setting_file_path)?;
       
        setting_file.write_all(yaml_string.as_bytes())?;

        Ok(())
    }
}

impl Serialize for YamlSetting {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Setting", 3)?;
        s.serialize_field("docker_container_id", &self.docker_container_id)?;
        s.serialize_field("shell", &self.shell)?;
        s.serialize_field("commands", &self.commands)?;
        s.end()
    }
}

// mod tests {
//     use super::*;

//     #[test]
//     fn test_read() {
//         let yaml_reader = YamlReader::new();
//         let from = 
// "docker_container_id: abcd
// shell: bash
// commands:
//   - ls
//   - cat";
//         let yaml_setting = Setting{
//             docker_container_id: String::from("abcd"),
//             shell: Shell::Bash,
//             commands: vec![String::from("ls"), String::from("cat")],
//         };
//         let parsed_yaml_setting = yaml_reader.read(from.as_bytes()).unwrap();

//         assert_eq!(parsed_yaml_setting, yaml_setting);
//     }

//     #[test]
//     fn test_write() {
//         let yaml_writer = YamlWriter::new();
//         let setting = Setting{
//             docker_container_id: String::from("abcd"),
//             shell: Shell::Bash,
//             commands: vec![String::from("ls"), String::from("cat")],
//         };
//         let to =
// "---
// docker_container_id: abcd
// shell: bash
// commands:
//   - ls
//   - cat
// ";
        
//         let mut writed_buf: Vec<u8> = Vec::new();
//         yaml_writer.write(&mut writed_buf, &setting).unwrap();
//         let yaml_writed_string = str::from_utf8(&writed_buf).unwrap();

//         assert_eq!(to, yaml_writed_string);
//     }

// //     #[test]
// //     fn test_read_unexpected_shell() {
// //         let yaml_reader = YamlReader::new();
// //         let from = 
// // "docker_container_id: abcd
// // shell: hogehoge
// // commands:
// //     - ls
// //     - cat";
// //         let parsed_yaml_setting = yaml_reader.read(from.as_bytes());
// //         match parsed_yaml_setting {
// //             Ok(_) => panic!(),
// //             Err(e) => assert_eq!(e, Error::UnExpectedShell)
// //         }
// //     }
// }
