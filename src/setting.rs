use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    SettingReadError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SettingReadError => write!(f, "setting: setting read error"),
        }
    }
}

impl error::Error for Error {}

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
    /// コンストラクタ
    /// 
    /// 引数
    /// 1. 設定ファイルへRead Writeを行うハンドラモジュール
    pub fn new (
        reader_writer: RW,
    ) -> SettingHandler<RW> {
        SettingHandler {
            reader_writer: reader_writer,
            setting: None,
        }
    }

    /// `SettingHandler`の`Setting`を初期化する関数
    /// 
    /// 引数
    /// 1. DockerコンテナID
    /// 2. `Shell`
    /// 3. デバックコンテナ内で利用するコマンド(TODO)
    pub fn init(
        &mut self,
        docker_container_id: &str,
        shell: Shell,
        commands: &[String],
    ) {
        let setting = Setting::new(docker_container_id, shell, commands);
        self.setting = Some(setting);
    }

    /// 設定ファイルに書き込む関数
    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref setting) = self.setting {
            self.reader_writer.write(setting)?;
        }

        Ok(())
    }

    /// 設定ファイルから読み込む関数
    /// 
    /// 読み込んだ`Setting`のイミュータブル参照を返却
    pub fn read(
        &mut self,
    ) -> Result<&Setting, Box<dyn std::error::Error>> {

        if let Some(ref setting) = self.setting {
            return Ok(setting)
        }

        self.setting = Some(self.reader_writer.read()?);
        if let Some(ref setting) = self.setting {
            return Ok(setting)
        }
        Err(Error::SettingReadError)?
    }

    /// 設定ファイルから読み込む関数
    /// 
    /// 読み込んだ`Setting`のミュータブル参照を返却
    pub fn read_mut(
        &mut self,
    ) -> Result<&mut Setting, Box<dyn std::error::Error>> {
        if let Some(ref mut setting) = self.setting {
            return Ok(setting)
        }

        self.setting = Some(self.reader_writer.read()?);
        if let Some(ref mut setting) = self.setting {
            return Ok(setting)
        }
        Err(Error::SettingReadError)?
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
        commands: &[String],
    ) -> Self {
        let commands: Vec<String> = commands.iter().map(|command| command.clone()).collect();

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