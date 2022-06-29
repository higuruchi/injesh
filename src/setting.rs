use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Shell {
    Bash,
    Sh,
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Shell::Bash => write!(f, "bash"),
            Shell::Sh => write!(f, "sh"),
        }
    }
}

pub trait Reader {
    fn read(&self) -> Result<Setting, Box<dyn std::error::Error>>;
}

pub trait Writer {
    fn write(
        &self,
        setting: &Setting,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Debug, PartialEq)]
pub struct SettingHandler<RW: Reader + Writer> {
    reader_writer: RW,
    setting: Option<Setting>,
}

impl<RW: Reader + Writer> SettingHandler<RW> {
    pub fn new (
        docker_container_id: &str,
        shell: Shell,
        commands: Vec<String>,
        reader_writer: RW,
    ) -> SettingHandler<RW> {
        let setting = Setting::new(docker_container_id, shell, commands);
        SettingHandler {
            reader_writer: reader_writer,
            setting: Some(setting),
        }
    }

    fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref setting) = self.setting {
            self.reader_writer.write(setting)?;
        }

        Ok(())
    }

    fn read(
        &mut self,
    ) -> Result<&Option<Setting>, Box<dyn std::error::Error>> {
        match self.setting {
            Some(ref setting) => Ok(&self.setting),
            None => {
                let setting = self.reader_writer.read()?;
                self.setting = Some(setting);
                Ok(&self.setting)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Setting {
    docker_container_id: String,
    shell: Shell,
    commands: Vec<String>,
}

impl Setting {
    pub fn new(
        docker_container_id: &str,
        shell: Shell,
        commands: Vec<String>,
    ) -> Self {
        Setting {
            docker_container_id: docker_container_id.to_string(),
            shell: shell,
            commands: commands
        }
    }

    pub fn docker_container_id(&self) -> &str {
        &self.docker_container_id
    }

    pub fn commands(&self) -> &[String] {
        &self.commands
    }

    pub fn shell(&self) -> Shell {
        self.shell
    }
}