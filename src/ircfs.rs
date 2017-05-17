extern crate libc;
use libc::{ENOENT, ENOSYS};

extern crate fuse;
// use fuse::{Filesystem, Request, ReplyDirectory};
use fuse::*;

extern crate time;
use time::Timespec;

use std::path::{Path, PathBuf};
use std::ffi::{OsStr, OsString};
use std::mem;
use std::collections::HashMap;
use std::os::raw::c_int;

pub struct IrcFs {
    directories: HashMap<u64, FuseDir>,
    highest_inode: u64,
}

// Key = Directory inode
// Value = Vec of inodes of contents of directory
pub struct NewIrcFs {
    files: HashMap<u64, Vec<u64>>,
}

pub struct FuseDir {
    name: OsString,
}

impl IrcFs {
    pub fn new() -> Self {
        IrcFs {
            directories: HashMap::new(),
            highest_inode: 1,
        }
    }

    pub fn insert_dir(&mut self, dir: FuseDir) {
        self.directories.insert(self.highest_inode + 1, dir);
        self.highest_inode += 3;
    }
}

impl Filesystem for IrcFs {
    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        let foo = FuseDir { name: OsString::from("foo") };
        self.insert_dir(foo);

        // let config = Config {
        //     nickname: Some("riiir-nickname".to_string()),
        //     username: Some("riiir-username".to_string()),
        //     realname: Some("riiir-realname".to_string()),
        //     server: Some("irc.rizon.net".to_string()),
        //     channels: Some(vec![
        //         "#cosarara".to_string(),
        //         "#riiir".to_string(),
        //     ]),
        //     .. Default::default()
        // };

        // thread::spawn(|| {
        //     match IrcServer::from_config(config) {
        //         Ok(server) => {
        //             server.identify();
        //             for message in server.iter() {
        //                 // Do something...eventually
        //             }
        //         },
        //         Err(e) => {
        //             println!("Error: {}", e);
        //         },
        //     };
        // });

        Ok(())
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr(ino={})", ino);
        let mut attr: FileAttr = unsafe { mem::zeroed() };
        // attr.perm = USER_DIR;
        let ttl = Timespec::new(1, 0);
        if ino == 1 {
            attr.ino = 1;
            attr.kind = FileType::Directory;
            reply.attr(&ttl, &attr);
        } else {
            match self.directories.get(&ino) {
                Some(file) => {
                    attr.ino = ino;
                    // attr.kind =
                    reply.attr(&ttl, &attr);
                },
                None => {
                    reply.error(ENOSYS);
                },
            }
        }
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup(parent={}, name={})", parent, name.to_string_lossy());
        match self.directories.get(&ino) {
            Some(file) => {
            },
            None => {
                reply.error(ENOSYS);
            },
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, fh: u64, offset: u64, mut reply: ReplyDirectory) {
        println!("readdir(ino={})", ino);

        if offset == 0 {
            reply.add(1, 0, FileType::Directory, ".");
            reply.add(1, 1, FileType::Directory, "..");

            if ino == 1 {
                for (ino, file) in &self.directories {
                    reply.add(ino.clone(), ino.clone(), FileType::Directory, &file.name);
                }
                reply.ok();
            } else {
                match self.directories.get(&ino) {
                    Some(file) => {
                        reply.add(ino.clone()+1, ino.clone()+1, FileType::RegularFile, "in");
                        reply.add(ino.clone()+2, ino.clone()+2, FileType::RegularFile, "out");
                        reply.ok();
                    },
                    None => {
                        reply.error(ENOSYS);
                    },
                }
            }
        }
    }
}
