use std::env::args_os;
use std::path::{Path, PathBuf};

fn main() {
    // Replace this with clap later
    let mountpoint = PathBuf::from(args_os().nth(1).expect("No mountpoint specified"));

    mount_fs(&mountpoint);
}

// Use separate function to eventually daemonize with some library
fn mount_fs<P: AsRef<Path>>(mountpoint: P) {
    unimplemented!();
}
