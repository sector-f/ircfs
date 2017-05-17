extern crate libc;
use libc::{ENOENT, ENOSYS};

extern crate fuse;
use fuse::*;

use std::ffi::OsStr;

pub struct IrcFs;

impl IrcFs {
    pub fn new() -> Self {
        IrcFs
    }
}

impl Filesystem for IrcFs {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup(parent={}, name={})", parent, name.to_string_lossy());
        reply.error(ENOSYS);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr(ino={})", ino);
        reply.error(ENOSYS);
    }

    fn readdir(&mut self, _req:&Request, ino:u64, fh:u64, offset:u64, mut reply:ReplyDirectory) {
        println!("readdir(ino={}, fh={}, offset={})", ino, fh, offset);
        reply.error(ENOSYS);
    }
}
