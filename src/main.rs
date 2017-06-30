extern crate serde;
#[macro_use] extern crate serde_derive;

extern crate fuse_mt;
use fuse_mt::FuseMT;

extern crate clap;
use clap::{App, Arg, AppSettings};

extern crate irc;
use irc::client::prelude::Config;

// extern crate daemonize;
// use daemonize::Daemonize;

extern crate toml;
extern crate time;
extern crate libc;
extern crate num_cpus;

use std::env::{current_dir, var_os};
use std::process::exit;
use std::path::PathBuf;
use std::fs::File;
use std::ffi::{OsStr, OsString};
use std::io::{stderr, Read, Write};

pub mod ircfs;
use ircfs::*;

pub mod config;
use config::*;

pub mod filesystem;
pub mod permissions;

fn is_valid_u16(n: &OsStr) -> Result<(), OsString> {
    let n = n.to_string_lossy();
    match n.parse::<u16>() {
        Ok(_) => {
            Ok(())
        },
        Err(_) => {
            Err(OsString::from("Must be a number from 1-65535"))
        },
    }
}

fn is_not_empty(n: &OsStr) -> Result<(), OsString> {
    if n.is_empty() {
        Err(OsString::from("Cannot be empty"))
    } else {
        Ok(())
    }
}

fn main() {
    let matches = App::new("ircfs")
        .arg(Arg::with_name("server")
             .value_name("SERVER")
             .short("s")
             .help("The server to connect to")
             .takes_value(true))
        .arg(Arg::with_name("directory")
             .value_name("MOUNTPOINT")
             .index(1)
             .help("The directory to mount the filesystem to")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("nickname")
             .value_name("NICKNAME")
             .short("n")
             .validator_os(is_not_empty)
             .help("Lets you override the default nickname. Default: $USER")
             .takes_value(true))
        .arg(Arg::with_name("port")
             .value_name("NUM")
             .short("p")
             .help("The port number to connect to. Default: 6667 without SSL, 6697 with SSL")
             .validator_os(is_valid_u16)
             .takes_value(true))
        .arg(Arg::with_name("pass_var")
             .value_name("ENV VAR")
             .short("k")
             .help("Lets you specify an environment variable containing your IRC password")
             .takes_value(true))
        .arg(Arg::with_name("realname")
             .value_name("REALNAME")
             .short("f")
             .help("Lets you override the default realname. Default: $USER")
             .takes_value(true))
        .arg(Arg::with_name("config")
             .value_name("FILE")
             .help("Specify path to config file")
             .short("c")
             .takes_value(true))
        .arg(Arg::with_name("ssl")
             .help("Connect via SSL")
             .long("ssl"))
        // .arg(Arg::with_name("daemonize")
        //      .short("d")
        //      .long("daemonize"))
        .setting(AppSettings::DeriveDisplayOrder)
        .get_matches();

    let mut config: Config = {
        match matches.value_of_os("config") {
            Some(path) => {
                match File::open(path) {
                    Ok(mut file) => {
                        let mut buf = String::new();
                        let _ = file.read_to_string(&mut buf);
                        match toml::from_str(&buf) {
                            Ok(config) => {
                                convert_config(config)
                            },
                            Err(e) => {
                                let _ = writeln!(stderr(),
                                    "Error parsing config file; falling back to defaults: {}", e
                                );
                                Default::default()
                            },
                        }
                    },
                    Err(e) => {
                        let _ = writeln!(stderr(),
                            "Error reading config file; falling back to defaults: {}", e
                        );
                        Default::default()
                    },
                }
            },
            None => {
                Default::default()
            },
        }
    };

    let nickname = matches
        .value_of_os("nickname").map(|s| s.to_owned())
        .or(var_os("USER"));

    if let Some(n) = nickname {
        config.nickname = Some(n.to_string_lossy().into_owned())
    }

    if let None = config.nickname {
        let _ = writeln!(stderr(), "nickname may not be unspecified");
        exit(1);
    }

    let realname = matches
        .value_of_os("realname").map(|s| s.to_owned())
        .or(var_os("USER")).unwrap_or(OsString::from(""));
    let realname = realname.to_string_lossy().into_owned();

    if let Some(s) = matches.value_of_os("server") {
        config.server = Some(s.to_string_lossy().into_owned());
    }

    if let None = config.server {
        let _ = writeln!(stderr(), "server may not be unspecified");
        exit(1);
    }

    config.realname = Some(realname);
    if let Some(port) = matches.value_of_os("port") {
        let port = port.to_string_lossy().into_owned();
        config.port = Some(port.parse::<u16>().unwrap());
    }
    if let Some(env_var) = matches.value_of_os("pass_var") {
        config.password = var_os(
            env_var.to_string_lossy().into_owned())
            .map(|s| s.to_string_lossy().into_owned());
    }
    config.use_ssl = Some(matches.is_present("ssl"));

    let num_threads = num_cpus::get();

    let mut mountpoint = PathBuf::from(matches.value_of_os("directory").unwrap());

    if mountpoint.is_relative() {
        let mut current_directory = match current_dir() {
            Ok(dir) => dir,
            Err(_) => {
                let _ = writeln!(stderr(), "Failed to determine current directory; try again with an absolute path");
                exit(1);
            },
        };
        current_directory.push(mountpoint);
        mountpoint = current_directory;
    }

    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };

    match IrcFs::new(&config, uid, gid){
        Ok(filesystem) => {
            let fuse_mt = FuseMT::new(filesystem, num_threads);
            if let Err(e) = fuse_mt::mount(fuse_mt, &mountpoint, &[]) {
                let _ = writeln!(stderr(), "Failed to mout filesystem: {}", e);
                exit(1);
            }
        },
        Err(e) => {
            let _ = writeln!(stderr(), "Failed to connect to IRC server: {}", e);
            exit(1);
        },
    }

    // if matches.is_present("daemonize") {
    //     let daemon = Daemonize::new()
    //         .privileged_action(move || {
    //             let filesystem = IrcFs::new(&config, uid, gid);
    //             let fuse_mt = FuseMT::new(filesystem, num_threads);
    //             let _ = fuse_mt::mount(fuse_mt, &mountpoint, &[]);
    //         });

    //     let _ = daemon.start();
    // } else {
    //     let filesystem = IrcFs::new(&config, uid, gid);
    //     let fuse_mt = FuseMT::new(filesystem, num_threads);
    //     let _ = fuse_mt::mount(fuse_mt, &mountpoint, &[]);
    // }
}
