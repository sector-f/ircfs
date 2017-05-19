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

pub mod ircfs;
use ircfs::*;

fn main() {
    let matches = App::new("riiir")
        .arg(Arg::with_name("mountpoint")
             .value_name("PATH")
             .required(true)
             .index(1))
        .arg(Arg::with_name("config")
             .value_name("FILE")
             .help("Specify path to config file")
             .short("c")
             .long("config")
             .takes_value(true))
        .arg(Arg::with_name("daemonize")
             .short("d")
             .long("daemonize"))
        .get_matches();

    // Replace this with clap later
    let mut mountpoint = PathBuf::from(matches.value_of_os("mountpoint").unwrap());

    if mountpoint.is_relative() {
        let mut current_directory = match current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                println!("Could not determine current directory: {}", e);
                exit(1);
            },
        };
        current_directory.push(mountpoint);
        mountpoint = current_directory;
    }

    if matches.is_present("daemonize") {
        let daemon = Daemonize::new()
            .privileged_action(move || {
                let _ = fuse::mount(IrcFs::new(), &mountpoint, &[]);
            });

        let _ = daemon.start();
    } else {
        let _ = fuse::mount(IrcFs::new(), &mountpoint, &[]);
    }
}
