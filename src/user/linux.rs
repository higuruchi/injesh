use crate::setting::Shell;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{env, error, fmt};

#[derive(Debug)]
pub enum Error {
    SudoUserNotFound,
    InvalidPasswd,
    InvalidUserId,
    InvalidGroupId,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SudoUserNotFound => write!(f, "user: sudo user not found"),
            Error::InvalidPasswd => write!(f, "user: invalid passwd"),
            Error::InvalidUserId => write!(f, "user: invalid userid"),
            Error::InvalidGroupId => write!(f, "user: invalid groupid"),
        }
    }
}

impl error::Error for Error {}

/// `/etc/passwd`の1行を表す
struct Passwd {
    user_name: String,
    password: String,
    user_id: u64,
    group_id: u64,
    comment: String,
    home_dir: String,
    login_shell: Option<Shell>,
}

impl Passwd {
    /// `/etc/passwd`の1行をパースする
    fn parse_passwd_line(line: &str) -> Result<Passwd, Box<dyn std::error::Error>> {
        let passwd_content: Vec<&str> = line.split(':').collect();
        let login_shell =
            if passwd_content[6] == "/usr/sbin/nologin" || passwd_content[6] == "/usr/bin/false" {
                None
            } else {
                match passwd_content[6] {
                    "/bin/bash" => Some(Shell::Bash),
                    "/bin/sh" => Some(Shell::Sh),
                    _ => None,
                }
            };

        let passwd = Passwd {
            user_name: passwd_content[0].to_string(),
            password: passwd_content[1].to_string(),
            // TODO: 適切なエラー型に変換
            user_id: passwd_content[2].parse()?,
            group_id: passwd_content[3].parse()?,
            comment: passwd_content[4].to_string(),
            home_dir: passwd_content[5].to_string(),
            login_shell: login_shell,
        };

        Ok(passwd)
    }

    fn parse_passwd() -> Result<Vec<Passwd>, Box<dyn std::error::Error>> {
        let passwd_path = "/etc/passwd";
        let passwd_file = File::open(passwd_path)?;
        let passwd_reader = BufReader::new(&passwd_file);
        let mut ret = Vec::new();
        for passwd_line in passwd_reader.lines() {
            let passwd_line = passwd_line?;
            let passwd = Self::parse_passwd_line(&passwd_line)?;
            ret.push(passwd);
        }

        Ok(ret)
    }
}

#[cfg(target_os = "linux")]
/// ユーザのホームディレクトリを`/etc/passwd`から検索し、`~/.injesh`を返却する
pub fn injesh_home_dir() -> Result<String, Box<dyn std::error::Error>> {
    let sudo_user = match env::var("SUDO_USER") {
        Ok(sudo_user) => sudo_user,
        Err(_) => match env::var("USER") {
            Ok(user) => user,
            Err(_) => Err(Error::SudoUserNotFound)?,
        },
    };

    let passwd = Passwd::parse_passwd()?;
    for p in passwd {
        if p.user_name == sudo_user {
            return Ok(format!("{}/{}", p.home_dir.clone(), ".injesh"));
        }
    }

    Err(Error::SudoUserNotFound)?
}
