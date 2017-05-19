extern crate libc;
use libc::{ENOENT, ENOSYS};

extern crate fuse;
// use fuse::{Filesystem, Request, ReplyDirectory};
use fuse::*;

extern crate irc;
use irc::client::prelude::*;

extern crate time;
use time::Timespec;

use std::ffi::{OsStr, OsString};
use std::collections::HashMap;
use std::os::raw::c_int;

pub struct IrcFs {
    files: HashMap<u64, IrcDir>, // Maps dir inodes to servers
    dir_map: HashMap<u64, u64>, // Maps in/out inodes to dir inodes
    attr: FileAttr, // Attributes for filesystem root dir
    highest_inode: u64,
}

struct IrcDir {
    name: OsString,
    // server: IrcServer,
    attr: FileAttr,
    infile: IrcFile,
    outfile: IrcFile,
    buf: Vec<u8>,
}

impl IrcDir {
    fn new(name: OsString, ino: u64, infile: IrcFile, outfile: IrcFile) -> Self {
        let init_time = time::get_time();

        let attr = FileAttr {
            ino: ino,
            size: 4096,
            blocks: 8,
            atime: init_time,
            mtime: init_time,
            ctime: init_time,
            crtime: init_time,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
        };

        IrcDir {
            name: name,
            attr: attr,
            infile: infile,
            outfile: outfile,
            buf: Vec::new(),
        }
    }
}

pub struct IrcFile {
    attr: FileAttr,
}

impl IrcFile {
    fn new(ino: u64,) -> Self {
        let init_time = time::get_time();

        let attr = FileAttr {
            ino: ino,
            size: 0,
            blocks: 0,
            atime: init_time,
            mtime: init_time,
            ctime: init_time,
            crtime: init_time,
            kind: FileType::RegularFile,
            perm: 0o644,
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
        };

        IrcFile { attr: attr }
    }
}

impl IrcFs {
    pub fn new() -> Self {
        let init_time = time::get_time();

        let attr = FileAttr {
            ino: 1,
            size: 4096,
            blocks: 8,
            atime: init_time,
            mtime: init_time,
            ctime: init_time,
            crtime: init_time,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2, // Number of hard links?
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
        };

        IrcFs {
            files: HashMap::new(),
            dir_map: HashMap::new(),
            attr: attr,
            highest_inode: 1,
        }
    }

    pub fn add_server(&mut self, name: OsString) {
        let ino = self.highest_inode;
        let infile = IrcFile::new(ino+2);
        let outfile = IrcFile::new(ino+3);
        let dir = IrcDir::new(name, ino+1, infile, outfile);
        self.files.insert(ino+1, dir);
        self.dir_map.insert(ino+2, ino+1);
        self.dir_map.insert(ino+3, ino+1);
        self.highest_inode += 3;
        self.attr.nlink += 1;
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

    fn getattr(&mut self, _req: &Request, req_ino: u64, reply: ReplyAttr) {
        println!("getattr(ino={})", req_ino);
        let ttl = Timespec::new(1, 0);
        if req_ino == 1 {
            reply.attr(&ttl, &self.attr);
            return;
        } else {
            for (dir_ino, dir) in &self.files {
                if &req_ino == dir_ino {
                    reply.attr(&ttl, &dir.attr);
                    return;
                } else if req_ino == dir.infile.attr.ino {
                    reply.attr(&ttl, &dir.infile.attr);
                    return;
                } else if req_ino == dir.outfile.attr.ino {
                    reply.attr(&ttl, &dir.outfile.attr);
                    return;
                }
            }
        }
        reply.error(ENOENT);
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let ttl = Timespec::new(1, 0);
        if parent == 1 {
            for (inode, dir) in &self.files {
                if name == dir.name {
                    reply.entry(&ttl, &self.files[&inode].attr, 0);
                    return;
                }
            }
        } else {
            for (inode, dir) in &self.files {
                if &parent == inode {
                    if name == "in" {
                        reply.entry(&ttl, &dir.infile.attr, 0);
                        return;
                    } else if name == "out" {
                        reply.entry(&ttl, &dir.outfile.attr, 0);
                        return;
                    } else {
                        reply.error(ENOENT);
                        return;
                    }

                }
            }
        }
        reply.error(ENOENT);
    }

    fn readdir(&mut self, _req:&Request, ino:u64, _fh:u64, offset:u64, mut reply:ReplyDirectory) {
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
                    Some(dir) => {
                        reply.add(1, 0, FileType::Directory, ".");
                        reply.add(1, 1, FileType::Directory, "..");

                        reply.add(
                            dir.infile.attr.ino,
                            dir.infile.attr.ino,
                            FileType::RegularFile,
                            "in",
                        );
                        reply.add(
                            dir.outfile.attr.ino,
                            dir.outfile.attr.ino,
                            FileType::RegularFile,
                            "out",
                        );

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

    fn read(&mut self,
        _req: &Request,
        req_ino: u64,
        fh: u64,
        offset: u64,
        size: u32,
        reply: ReplyData) {
        println!("read(ino={}, fh={}, offset={}, size={})", req_ino, fh, offset, size);

        if let Some(_dir) = self.dir_map.get(&req_ino) {
            reply.data(&"Hello, world!".as_bytes()[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
    }
}
