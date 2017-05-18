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
    inode_map: HashMap<u64, Vec<u64>>, // Maps dir inodes to file inodes
    directories: HashMap<u64, FuseDir>,
    highest_inode: u64,
    init_time: Timespec,
}

pub struct FuseDir {
    name: OsString,
}

impl IrcFs {
    pub fn new() -> Self {
        IrcFs {
            inode_map: HashMap::new(),
            directories: HashMap::new(),
            highest_inode: 1,
            init_time: time::now().to_timespec(),
        }
    }

    pub fn insert_dir(&mut self, dir: FuseDir) {
        let ino = self.highest_inode;
        self.directories.insert(ino+1, dir);
        self.inode_map.insert(ino+1, vec![ino+2, ino+3]);
        self.highest_inode += 3;
    }
}

impl Filesystem for IrcFs {
    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        let foo = FuseDir { name: OsString::from("foo") };
        let bar = FuseDir { name: OsString::from("bar") };
        let baz = FuseDir { name: OsString::from("baz") };

        self.insert_dir(foo);
        self.insert_dir(bar);
        self.insert_dir(baz);

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
        let ttl = Timespec::new(1, 0);
        if ino == 1 {
            let attr = FileAttr {
                ino: ino.clone(),
                size: 4096,
                blocks: 8,
                atime: self.init_time,
                mtime: self.init_time,
                ctime: self.init_time,
                crtime: self.init_time,
                kind: FileType::Directory,
                perm: 0o755,
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 0,
                flags: 0,
            };

            reply.attr(&ttl, &attr);
            return;
        } else {
            for (inode, dir) in &self.inode_map {
                let infile = &dir[0];
                let outfile = &dir[1];

                if &ino == inode {
                    let attr = FileAttr {
                        ino: ino.clone(),
                        size: 4096,
                        blocks: 8,
                        atime: self.init_time,
                        mtime: self.init_time,
                        ctime: self.init_time,
                        crtime: self.init_time,
                        kind: FileType::Directory,
                        perm: 0o755,
                        nlink: 1,
                        uid: 0,
                        gid: 0,
                        rdev: 0,
                        flags: 0,
                    };

                    reply.attr(&ttl, &attr);
                    return;
                } else if &ino == infile {
                    let attr = FileAttr {
                        ino: infile.clone(),
                        size: 0,
                        blocks: 0,
                        atime: self.init_time,
                        mtime: self.init_time,
                        ctime: self.init_time,
                        crtime: self.init_time,
                        kind: FileType::RegularFile,
                        perm: 0o644,
                        nlink: 1,
                        uid: 0,
                        gid: 0,
                        rdev: 0,
                        flags: 0,
                    };
                    reply.attr(&ttl, &attr);
                    return;
                } else if &ino == outfile {
                    let attr = FileAttr {
                        ino: outfile.clone(),
                        size: 0,
                        blocks: 0,
                        atime: self.init_time,
                        mtime: self.init_time,
                        ctime: self.init_time,
                        crtime: self.init_time,
                        kind: FileType::RegularFile,
                        perm: 0o644,
                        nlink: 1,
                        uid: 0,
                        gid: 0,
                        rdev: 0,
                        flags: 0,
                    };
                    reply.attr(&ttl, &attr);
                    return;
                }
            }
        }
        reply.error(ENOENT);
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup(parent={}, name={})", parent, name.to_string_lossy());
        let ttl = Timespec::new(1, 0);
        if parent == 1 {
            for (ino, dir) in &self.directories {
                if name == dir.name {
                    let attr = FileAttr {
                        ino: ino.clone(),
                        size: 4096,
                        blocks: 8,
                        atime: self.init_time,
                        mtime: self.init_time,
                        ctime: self.init_time,
                        crtime: self.init_time,
                        kind: FileType::Directory,
                        perm: 0o755,
                        nlink: 1,
                        uid: 0,
                        gid: 0,
                        rdev: 0,
                        flags: 0,
                    };

                    reply.entry(&ttl, &attr, 0);
                    return;
                }
            }
        } else {
            for (ino, dir) in &self.directories {
                let inode = {
                    if name == "in" {
                        &self.inode_map[ino][0]
                    } else if name == "out" {
                        &self.inode_map[ino][1]
                    } else {
                        reply.error(ENOENT);
                        return;
                    }
                };


                if &parent == ino {
                    if name == "in" || name == "out" {
                        let attr = FileAttr {
                            ino: inode.clone(),
                            size: 0,
                            blocks: 0,
                            atime: self.init_time,
                            mtime: self.init_time,
                            ctime: self.init_time,
                            crtime: self.init_time,
                            kind: FileType::RegularFile,
                            perm: 0o644,
                            nlink: 1,
                            uid: 0,
                            gid: 0,
                            rdev: 0,
                            flags: 0,
                        };
                        reply.entry(&ttl, &attr, 0);
                        return;
                    }
                }
            }
        }
        reply.error(ENOENT);
    }

    fn readdir(&mut self, _req:&Request, ino:u64, fh:u64, offset:u64, mut reply:ReplyDirectory) {
        println!("readdir(ino={})", ino);
        if offset == 0 {
            if ino == 1 {
                reply.add(1, 0, FileType::Directory, ".");
                reply.add(1, 1, FileType::Directory, "..");
                for (ino, dir) in &self.directories {
                    reply.add(ino.clone(), ino.clone(), FileType::Directory, &dir.name);
                }
                reply.ok();
                return;
            } else {
                match self.inode_map.get(&ino) {
                    Some(dir) => {
                        reply.add(1, 0, FileType::Directory, ".");
                        reply.add(1, 1, FileType::Directory, "..");
                        reply.add(dir[0], dir[0], FileType::RegularFile, "in");
                        reply.add(dir[1], dir[1], FileType::RegularFile, "out");
                        reply.ok();
                    },
                    None => {
                        reply.error(ENOENT);
                    },
                }
                return;
            }
        }
        reply.error(ENOENT);
    }
}
