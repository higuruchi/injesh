use crate::setting::{Reader, Setting, Shell, Writer};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::str;

use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::{error, fmt};

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
            Error::UnExpectedShell => write!(f, "setting_yaml: unexpected shell"),
            Error::UnExpectedCommand => write!(f, "setting_yaml: unexpected command"),
            Error::UnexpectedContainerId => write!(f, "setting_yaml: unexpected container id"),
            Error::Parse => write!(f, "setting_yaml: parse error"),
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

pub struct YamlReaderWriter {
    setting_file_path: PathBuf,
}

impl YamlReaderWriter {
    pub fn new(path: &Path) -> YamlReaderWriter {
        YamlReaderWriter {
            setting_file_path: path.to_path_buf(),
        }
    }
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
            &setting_yaml.commands,
        ))
    }
}

impl Writer for YamlReaderWriter {
    fn write(&self, setting: &Setting) -> Result<(), Box<dyn std::error::Error>> {
        let commands: Vec<String> = setting
            .commands()
            .iter()
            .map(|command| command.to_string())
            .collect();
        let yaml_setting = YamlSetting {
            docker_container_id: setting.docker_container_id().to_string(),
            shell: setting.shell().to_string(),
            commands: commands,
        };

        let yaml_string = serde_yaml::to_string(&yaml_setting)?;

        let mut setting_file = OpenOptions::new()
            .write(true)
            .create(true)
            .read(false)
            .open(&self.setting_file_path)?;

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

mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let setting_file_path = "/tmp/setting_read_test.yaml";
        let from = "---
docker_container_id: abcd
shell: bash
commands:
  - ls
  - cat
";
        let mut setting_file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(setting_file_path)
            .unwrap();
        setting_file.write_all(from.as_bytes()).unwrap();

        let yaml_rw = YamlReaderWriter::new(&PathBuf::from(setting_file_path));
        let commands = vec![String::from("ls"), String::from("cat")];
        let yaml_setting = Setting::new("abcd", Shell::Bash, &commands);
        let parsed_yaml_setting = yaml_rw.read().unwrap();

        assert_eq!(parsed_yaml_setting, yaml_setting);
    }

    #[test]
    fn test_write() {
        let setting_file_path = "/tmp/setting_write_test.yaml";
        let yaml_rw = YamlReaderWriter::new(&PathBuf::from(setting_file_path));
        let commands = vec![String::from("ls"), String::from("cat")];
        let yaml_setting = Setting::new("abcd", Shell::Bash, &commands);
        yaml_rw.write(&yaml_setting).unwrap();

        let to = "---
docker_container_id: abcd
shell: bash
commands:
  - ls
  - cat
";
        let mut setting_file = OpenOptions::new()
            .write(false)
            .read(true)
            .create(false)
            .open(setting_file_path)
            .unwrap();
        let mut write_test_buf: [u8; 67] = [0; 67];
        setting_file.read(&mut write_test_buf).unwrap();

        assert_eq!(to, str::from_utf8(&write_test_buf).unwrap());
    }

    //     #[test]
    //     fn test_read_unexpected_shell() {
    //         let yaml_reader = YamlReader::new();
    //         let from =
    // "docker_container_id: abcd
    // shell: hogehoge
    // commands:
    //     - ls
    //     - cat";
    //         let parsed_yaml_setting = yaml_reader.read(from.as_bytes());
    //         match parsed_yaml_setting {
    //             Ok(_) => panic!(),
    //             Err(e) => assert_eq!(e, Error::UnExpectedShell)
    //         }
    //     }
}
