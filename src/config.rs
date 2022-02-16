use clap::{App, Arg, SubCommand};

const INIT:                 &str = "init";
const LAUNCH:               &str = "launch";
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

pub fn parse() {
    let about_this_app = "Applications for debugging into containers without shells such as distroless and scratch. 
It is possible to enter a container by sharing namespaces \
such as cgroup, ipc, net, pid, user, uts, etc. with the container to be debugged. 
File operations performed in the debugging container do not affect the original container.";

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
            SubCommand::with_name(INIT)
            .about(init_about)
        )
        .subcommand(
            // launchサブコマンド
            SubCommand::with_name(LAUNCH)
            .about(launch_about)
            .arg(
                Arg::with_name(CONTAINER_ID_OR_NAME)
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name(OPT_ROOTFS)
                .long(OPT_ROOTFS)
                .takes_value(true)
            )
            .arg(
                Arg::with_name(OPT_ROOTFS_IMAGE)
                .long(OPT_ROOTFS_IMAGE)
                .takes_value(true)
            )
            .arg(
                Arg::with_name(OPT_ROOTFS_DOCKER)
                .long(OPT_ROOTFS_DOCKER)
                .takes_value(true)
            )
            .arg(
                Arg::with_name(OPT_ROOTFS_LXD)
                .long(OPT_ROOTFS_LXD)
                .takes_value(true)
            )
        )
        .subcommand(
            // deleteサブコマンド
            SubCommand::with_name(DELETE)
            .about(delete_about)
            .arg(
                Arg::with_name(CONTAINER_ID_OR_NAME)
                .takes_value(true)
                .required(true)
            )
        )
        .subcommand(
            // listサブコマンド
            SubCommand::with_name(LIST)
            .about(list_about)
        )
        .subcommand(
            // file関連サブコマンド
            SubCommand::with_name(FILE)
            .subcommand(
                // pullサブコマンド
                SubCommand::with_name(PULL)
                .about(file_pull_about)
                .arg(
                    Arg::with_name(CONTAINER_ID_OR_NAME)
                    .takes_value(true)
                )
                .arg(
                    Arg::with_name("from")
                    .takes_value(true)
                )
                .arg(
                    Arg::with_name("to")
                    .takes_value(true)
                )
            )
            .subcommand(
                // pushサブコマンド
                SubCommand::with_name(PUSH)
                .about(file_push_about)
                .arg(
                    Arg::with_name(CONTAINER_ID_OR_NAME)
                    .takes_value(true)
                )
                .arg(
                    Arg::with_name("from")
                    .takes_value(true)
                )
                .arg(
                    Arg::with_name("to")
                    .takes_value(true)
                )
            )
        );
    
    let matches = app.get_matches();
    
    // init解析ハンドラ
    if let Some(ref matches) = matches.subcommand_matches(INIT) {
        println!("used init");
    }

    // launch解析ハンドラ
    if let Some(ref matches) = matches.subcommand_matches(LAUNCH) {
        println!("used launch {:?}  {:?}", matches.value_of("Containter ID or name"), matches.value_of(OPT_ROOTFS));
    }

    // delete解析ハンドラ
    if let Some(ref matches) = matches.subcommand_matches(DELETE) {
        println!("used delete");
    }

    // list解析ハンドラ
    if let Some(ref matches) = matches.subcommand_matches(LIST) {
        println!("used list");
    }

    // file解析ハンドラ
    if let Some(ref matches) = matches.subcommand_matches(FILE) {
        println!("used file ");
        // // pull解析ハンドラ
        if let Some(ref matches) = matches.subcommand_matches(PULL){
            println!("used file pull");
        }

        // push解析ハンドラ
        if let Some(ref matches) = matches.subcommand_matches(PUSH) {
            println!("used file push")
        }
    }
}