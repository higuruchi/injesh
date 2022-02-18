use clap::{App, Arg};
use crate::command::{self, SubCommand, Exec, Launch, RootFSOption, FileSubCommand, File, Error};
use std::path::PathBuf;

const INIT:                 &str = "init";
const LAUNCH:               &str = "launch";
const EXEC:                 &str = "exec";
const NAME:                 &str = "name";
const CMD:                  &str = "cmd";
const DELETE:               &str = "delete";
const LIST:                 &str = "list";
const FILE:                 &str = "file";
const PULL:                 &str = "pull";
const PUSH:                 &str = "push";
const OPT_ROOTFS:           &str = "--rootfs";
const OPT_ROOTFS_IMAGE:     &str = "--rootfs-image";
const OPT_ROOTFS_DOCKER:    &str = "--rootfs-docker";
const OPT_ROOTFS_LXD:       &str = "--rootfs-lxd";
const CONTAINER_ID_OR_NAME: &str = "Container ID or name";

pub fn parse() -> Result<command::SubCommand, command::Error> {
    let about_this_app = "Applications for debugging into containers without shells such as distroless and scratch. 
It is possible to enter a container by sharing namespaces \
such as cgroup, ipc, net, pid, user, uts, etc. with the container to be debugged. 
File operations performed in the debugging container do not affect the original container.";

    let exec_about = "Enter any existing container and run CMD.
If there is no CMD, invoke the shell in the configuration value file.";

    let init_about = "Initialize configuration files, .injesh directory, etc.
Run only once after installing injesh.";

    let launch_about = "Create a new debug container and get inside the debug container (mount overlayfs, etc.)
Get the executable files of the commands and dependent libraries described in the configuration file, create a rootfs, and then start it (pending).
NAME is the name of the debug container. If it is not specified, it will be generated automatically.
If CMD is not specified, the default shell is used.";

    let delete_about = "Remove the debug container";
    let list_about = "List debug containers";
    let file_pull_about = "Download the specified file of the debug container.";
    let file_push_about = "Uploading the specified file of the host to the specified PATH of the debug container";

    let app = App::new("injesh")
        .version("0.0.0")
        .author("higuruchi <hfumiya2324@gmail.com>")
        .about(about_this_app)
        .subcommand(
            App::new(INIT)
            .about(init_about)
        )
        .subcommand(
            // launchサブコマンド
            App::new(LAUNCH)
            .about(launch_about)
            .arg(
                Arg::new(CONTAINER_ID_OR_NAME)
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::new(OPT_ROOTFS)
                .long(OPT_ROOTFS)
                .takes_value(true)
            )
            .arg(
                Arg::new(OPT_ROOTFS_IMAGE)
                .long(OPT_ROOTFS_IMAGE)
                .takes_value(true)
            )
            .arg(
                Arg::new(OPT_ROOTFS_DOCKER)
                .long(OPT_ROOTFS_DOCKER)
                .takes_value(true)
            )
            .arg(
                Arg::new(OPT_ROOTFS_LXD)
                .long(OPT_ROOTFS_LXD)
                .takes_value(true)
            )
            .arg(
                Arg::new(NAME)
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::new(CMD)
                .takes_value(true)
            )

        )
        .subcommand(
            // deleteサブコマンド
            App::new(DELETE)
            .about(delete_about)
            .arg(
                Arg::new(NAME)
                .takes_value(true)
                .required(true)
            )
        )
        .subcommand(
            // listサブコマンド
            App::new(LIST)
            .about(list_about)
        )
        .subcommand(
            // file関連サブコマンド
            App::new(FILE)
            .subcommand(
                // pullサブコマンド
                App::new(PULL)
                .about(file_pull_about)
                .arg(
                    Arg::new("from")
                    .takes_value(true)
                )
                .arg(
                    Arg::new("to")
                    .takes_value(true)
                )
            )
            .subcommand(
                // pushサブコマンド
                App::new(PUSH)
                .about(file_push_about)
                .arg(
                    Arg::new("from")
                    .takes_value(true)
                )
                .arg(
                    Arg::new("to")
                    .takes_value(true)
                )
            )
        )
        .subcommand(
            App::new(EXEC)
            .about(exec_about)
            .arg(
                Arg::new(NAME)
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::new(CMD)
                .takes_value(true)
            )
        );
    
    let matches = app.get_matches();

    match matches.subcommand() {
        Some((INIT, _))     => {
            Ok(SubCommand::Init)
        },
        Some((LAUNCH, sub_m))   => {
            let container = match sub_m.value_of(CONTAINER_ID_OR_NAME) {
                Some(container) => container,
                None => return Err(Error::CommandError)
            };
            let opt_rootfs = sub_m.value_of(OPT_ROOTFS);
            let opt_rootfs_image = sub_m.value_of(OPT_ROOTFS_IMAGE);
            let opt_rootfs_docker = sub_m.value_of(OPT_ROOTFS_DOCKER);
            let opt_rootfs_lxd = sub_m.value_of(OPT_ROOTFS_LXD);
            let name = match sub_m.value_of(NAME) {
                Some(name) => name,
                None => return Err(Error::CommandError)
            };
            let cmd = match sub_m.value_of(CMD) {
                Some(cmd) => Some(String::from(cmd)),
                None => None
            };
            let rootfs = check_rootfs(opt_rootfs, opt_rootfs_image, opt_rootfs_docker, opt_rootfs_lxd)?;

            Ok(SubCommand::Launch(Launch::new(
                String::from(container),
                rootfs,
                String::from(name),
                cmd
            )))
        },
        Some((EXEC, sub_m)) => {
            let name = match sub_m.value_of(NAME) {
                Some(name) => String::from(name),
                None => return Err(Error::CommandError)
            };
            let cmd = match sub_m.value_of(CMD) {
                Some(cmd) => Some(String::from(cmd)),
                None => None
            };

            Ok(SubCommand::Exec(Exec::new(name, cmd)))
        },
        Some((DELETE, sub_m)) => {
            let container = match sub_m.value_of(NAME) {
                Some(container) => String::from(container),
                None => return Err(command::Error::CommandError)
            };

            Ok(SubCommand::Delete(container))
        },
        Some((LIST, _)) => {
            Ok(SubCommand::List)
        },
        Some((FILE, sub_m)) => {
            match sub_m.subcommand() {
                Some((PULL, sub_m)) => {
                    let name_and_path = match sub_m.value_of("from") {
                        Some(from) => match parse_container_path(from) {
                            Ok(name_and_path) => name_and_path,
                            Err(_) => return Err(Error::CommandError)
                        },
                        None => return Err(Error::CommandError)
                    };
                    let to = match sub_m.value_of("to") {
                        Some(to) => String::from(to),
                        None => return Err(Error::CommandError)
                    };

                    Ok(
                        SubCommand::File(
                            FileSubCommand::Pull(
                                File::new(String::from(name_and_path.0), PathBuf::from(name_and_path.1), PathBuf::from(to))
                            )
                        )
                    )
                },
                Some((PUSH, sub_m)) => {
                    let name_and_path = match sub_m.value_of("from") {
                        Some(from) => match parse_container_path(from) {
                            Ok(name_and_path) => name_and_path,
                            Err(_) => return Err(Error::CommandError)
                        },
                        None => return Err(Error::CommandError)
                    };
                    let to = match sub_m.value_of("to") {
                        Some(to) => String::from(to),
                        None => return Err(Error::CommandError)
                    };

                    Ok(
                        SubCommand::File(
                            FileSubCommand::Pull(
                                File::new(String::from(name_and_path.0), PathBuf::from(name_and_path.1), PathBuf::from(to))
                            )
                        )
                    )
                },
                _ => {
                    Err(Error::CommandError)
                }
            }
        },
        _ => {
            Err(Error::CommandError)
        }
    }

}

fn check_rootfs(
    opt_rootfs:         Option<&str>,
    opt_rootfs_image:   Option<&str>,
    opt_rootfs_docker:  Option<&str>,
    opt_rootfs_lxd:     Option<&str>
) -> Result<RootFSOption, command::Error> {
    let mut num_of_some = 0;
    let mut rootfs = command::RootFSOption::None;

    if opt_rootfs.is_some() {
        num_of_some += 1;
        rootfs = RootFSOption::Rootfs(String::from(opt_rootfs.unwrap_or("")));
    }
    if opt_rootfs_image.is_some() {
        num_of_some += 1;
        rootfs = RootFSOption::RootfsImage(String::from(opt_rootfs_image.unwrap_or("")));
    }
    if opt_rootfs_docker.is_some() {
        num_of_some += 1;
        rootfs = RootFSOption::RootfsDocker(String::from(opt_rootfs_docker.unwrap_or("")));
    }
    if opt_rootfs_lxd.is_some() {
        num_of_some += 1;
        rootfs = RootFSOption::RootfsLxd(String::from(opt_rootfs_lxd.unwrap_or("")));
    }

    if num_of_some > 1 {
        return Err(Error::CommandError)
    }

    Ok(rootfs)
}

fn parse_container_path(container_path: &str) -> Result<(&str, &str), Error> {
    let name_and_path: Vec<&str> = container_path.split(':').collect();

    if name_and_path.len() != 2 {
        return Err(Error::CommandError)
    }

    Ok((name_and_path[0], name_and_path[1]))
}