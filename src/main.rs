extern crate fuse;
use fuse::{Filesystem, Request};

use std::os::raw::c_int;
use std::env::args_os;

struct IrcFs;

impl Filesystem for IrcFs {
    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        unimplemented!();
    }
}

fn main() {
    // Replace this with clap later
    let mountpoint = args_os().nth(1).expect("No mountpoint specified");
}
