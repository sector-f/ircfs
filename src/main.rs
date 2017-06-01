extern crate serde;
extern crate toml;
extern crate time;
extern crate fuse;
extern crate irc;
extern crate libc;

extern crate clap;
use clap::{App, Arg};

extern crate daemonize;
use daemonize::Daemonize;

use std::env::current_dir;
use std::process::exit;
use std::path::PathBuf;
use std::fs::File;
use std::io::{stderr, Read, Write};

extern crate ircfs;
use ircfs::ircfs::*;

fn main() {
    let matches = App::new("ircfs")
        .arg(Arg::with_name("mountpoint")
             .value_name("PATH")
             .required(true)
             .index(1))
        .arg(Arg::with_name("config")
             .value_name("FILE")
             .help("Specify path to config file")
             .short("c")
             .long("config")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("daemonize")
             .short("d")
             .long("daemonize"))
        .get_matches();

    let config = match File::open(matches.value_of_os("config").unwrap()) {
        Ok(mut file) => {
            let mut buf = String::new();
            let _ = file.read_to_string(&mut buf);
            match toml::from_str(&buf) {
                Ok(config) => {
                    config
                },
                Err(e) => {
                    let _ = writeln!(stderr(), "Error parsing config file: {}", e);
                    exit(1);
                },
            }
        },
        Err(e) => {
            let _ = writeln!(stderr(), "Error reading config file: {}", e);
            exit(1);
        },
    };

    let mut mountpoint = PathBuf::from(matches.value_of_os("mountpoint").unwrap());

    if mountpoint.is_relative() {
        let mut current_directory = match current_dir() {
            Ok(dir) => dir,
            Err(_) => {
                println!("Failed to determine current directory to form absolute path to mountpoint; try again with an absolute path");
                exit(1);
            },
        };
        current_directory.push(mountpoint);
        mountpoint = current_directory;
    }

    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };

    if matches.is_present("daemonize") {
        let daemon = Daemonize::new()
            .privileged_action(move || {
                let _ = fuse::mount(IrcFs::new(&config, uid, gid), &mountpoint, &[]);
            });

        let _ = daemon.start();
    } else {
        let _ = fuse::mount(IrcFs::new(&config, uid, gid), &mountpoint, &[]);
    }
}
