extern crate serde;
extern crate toml;
extern crate time;
extern crate irc;
extern crate libc;
extern crate num_cpus;

extern crate fuse_mt;
use fuse_mt::FuseMT;

extern crate clap;
use clap::{App, Arg};

// extern crate daemonize;
// use daemonize::Daemonize;

use std::env::{current_dir, var_os};
use std::process::exit;
use std::path::PathBuf;
use std::fs::File;
use std::ffi::{OsStr, OsString};
use std::io::{stderr, Read, Write};

extern crate ircfs;
use ircfs::ircfs::*;
use ircfs::config::*;

fn main() {
    let matches = App::new("ircfs")
        .arg(Arg::with_name("server")
             .value_name("SERVER")
             .short("s")
             .help("The server to connect to. Default: chat.freenode.net")
             .takes_value(true))
        .arg(Arg::with_name("port")
             .value_name("NUM")
             .short("p")
             .help("The port number to connect to. Default: 6667 without SSL, 6697 with SSL")
             .takes_value(true))
        .arg(Arg::with_name("pass_var")
             .value_name("ENV VAR")
             .short("k")
             .help("Lets you specify an environment variable containing your IRC password")
             .takes_value(true))
        .arg(Arg::with_name("directory")
             .value_name("PATH")
             .short("i")
             .help("Lets you override the default IRC path. Default: ~/irc")
             .takes_value(true))
        .arg(Arg::with_name("nickname")
             .value_name("NICKNAME")
             .short("n")
             .takes_value(true))
        .arg(Arg::with_name("realname")
             .value_name("REALNAME")
             .short("f")
             .takes_value(true))
        .arg(Arg::with_name("config")
             .value_name("FILE")
             .help("Specify path to config file")
             .short("c")
             // .long("config")
             .takes_value(true))
        // .arg(Arg::with_name("daemonize")
        //      .short("d")
        //      .long("daemonize"))
        .get_matches();

    let server = matches.value_of_os("server")
        .unwrap_or(OsStr::new("irc.freenode.net"))
        .to_string_lossy();

    let nickname = matches
        .value_of_os("nickname").map(|s| s.to_owned())
        .or(var_os("USER")).unwrap_or(OsString::from(""));
    let nickname = nickname.to_string_lossy();

    if nickname.is_empty() {
        let _ = writeln!(stderr(), "nickname may not be empty");
        exit(1);
    }

    let num_threads = num_cpus::get();

    let config = match File::open(matches.value_of_os("config").unwrap()) {
        Ok(mut file) => {
            let mut buf = String::new();
            let _ = file.read_to_string(&mut buf);
            match toml::from_str(&buf) {
                Ok(config) => {
                    convert_config(config)
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
            let _ = fuse_mt::mount(fuse_mt, &mountpoint, &[]);
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
