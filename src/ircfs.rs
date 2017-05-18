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
    files: HashMap<u64, FuseServer>,
    highest_inode: u64,
    init_time: Timespec,
}

pub struct FuseServer {
    // ino: u64,
    name: OsString,
    files: Vec<FuseFile>,
}

pub enum FuseFile {
    InFile(u64),
    OutFile(u64),
}

impl FuseFile {
    pub fn ino(&self) -> u64 {
        match *self {
            FuseFile::InFile(ino) => ino,
            FuseFile::OutFile(ino) => ino,
        }
    }
}

impl IrcFs {
    pub fn new() -> Self {
        IrcFs {
            files: HashMap::new(),
            highest_inode: 1,
            init_time: time::now().to_timespec(),
        }
    }

    pub fn add_server(&mut self, name: OsString) {
        let ino = self.highest_inode;
        let server = FuseServer {
            name: name,
            files: vec![FuseFile::InFile(ino+2), FuseFile::OutFile(ino+3)],
        };
        self.files.insert(ino+1, server);
        self.highest_inode += 3;
    }
}

impl Filesystem for IrcFs {
    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        let foo = OsString::from("foo");
        let bar = OsString::from("bar");
        let baz = OsString::from("baz");

        self.add_server(foo);
        self.add_server(bar);
        self.add_server(baz);

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
            for (inode, server) in &self.files {
                let infile = &server.files[0].ino();
                let outfile = &server.files[1].ino();

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
            for (inode, server) in &self.files {
                if name == server.name {
                    let attr = FileAttr {
                        ino: inode.clone(),
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
            for (inode, server) in &self.files {
                let in_inode = &self.files[inode].files[0].ino();
                let out_inode = &self.files[inode].files[1].ino();

                let reply_inode = {
                    if name == "in" {
                        in_inode.clone()
                    } else if name == "out" {
                        out_inode.clone()
                    } else {
                        reply.error(ENOENT);
                        return;
                    }
                };

                if &parent == inode {
                    if name == "in" || name == "out" {
                        let attr = FileAttr {
                            ino: reply_inode.clone(),
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
                for (inode, server) in &self.files {
                    reply.add(inode.clone(), inode.clone(), FileType::Directory, &server.name);
                }
                reply.ok();
                return;
            } else {
                match self.files.get(&ino) {
                    Some(server) => {
                        reply.add(1, 0, FileType::Directory, ".");
                        reply.add(1, 1, FileType::Directory, "..");
                        reply.add(server.files[0].ino(), server.files[0].ino(), FileType::RegularFile, "in");
                        reply.add(server.files[1].ino(), server.files[1].ino(), FileType::RegularFile, "out");
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
