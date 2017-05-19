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

#[derive(Debug)]
pub struct IrcFs {
    files: HashMap<u64, FuseFile>,
    attr: FileAttr, // Attributes for filesystem root dir
    highest_inode: u64,
}

#[derive(Debug)]
enum FuseFile {
    Dir(IrcDir),
    File(IrcFile),
}

#[derive(Debug)]
struct IrcDir {
    name: OsString,
    // server: IrcServer,
    parent: u64, // Probably always gonna be 1
    attr: FileAttr,
    in_inode: u64,
    out_inode: u64,
}

#[derive(Debug)]
pub struct IrcFile {
    parent: u64,
    attr: FileAttr,
    buf: Vec<u8>,
}

impl FuseFile {
    pub fn parent(&self) -> u64 {
        match *self {
            FuseFile::Dir(ref dir) => dir.parent,
            FuseFile::File(ref file) => file.parent,
        }
    }

    pub fn attr(&self) -> FileAttr {
        match *self {
            FuseFile::Dir(ref dir) => dir.attr,
            FuseFile::File(ref file) => file.attr,
        }
    }
}

impl IrcDir {
    fn new(name: OsString, ino: u64, in_inode: u64, out_inode: u64) -> Self {
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
            parent: 1, // Hard-code this (for now?)
            attr: attr,
            in_inode: in_inode,
            out_inode: out_inode,
        }
    }
}

impl IrcFile {
    fn new(ino: u64, parent: u64) -> Self {
        let init_time = time::get_time();

        let attr = FileAttr {
            ino: ino,
            size: 0,
            blocks: 1,
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

        IrcFile {
            parent: parent,
            attr: attr,
            buf: Vec::new(),
        }
    }

    fn insert_data(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
        self.attr.size += data.len() as u64;
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
            attr: attr,
            highest_inode: 1,
        }
    }

    pub fn add_server(&mut self, name: OsString) {
        let dir_ino = self.highest_inode + 1;
        let in_ino = dir_ino + 1;
        let out_ino = dir_ino + 2;

        let infile = IrcFile::new(in_ino, dir_ino);
        let mut outfile = IrcFile::new(out_ino, dir_ino);
        outfile.insert_data("Test data\n".as_bytes());
        let dir = IrcDir::new(name, dir_ino, in_ino, out_ino);

        self.files.insert(dir_ino, FuseFile::Dir(dir));
        self.files.insert(in_ino, FuseFile::File(infile));
        self.files.insert(out_ino, FuseFile::File(outfile));

        self.highest_inode += 3;
        self.attr.nlink += 1;
    }
}

impl Filesystem for IrcFs {
    fn init(&mut self, req: &Request) -> Result<(), c_int> {
        let foo = OsString::from("foo");
        let bar = OsString::from("bar");
        let baz = OsString::from("baz");

        self.add_server(foo);
        self.add_server(bar);
        self.add_server(baz);

        self.attr.uid = req.uid();
        self.attr.gid = req.gid();

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
        let ttl = Timespec::new(1, 0);
        if req_ino == 1 {
            reply.attr(&ttl, &self.attr);
            return;
        } else {
            if let Some(file) = self.files.get(&req_ino) {
                reply.attr(&ttl, &file.attr());
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let ttl = Timespec::new(1, 0);
        if parent == 1 {
            for (inode, file) in &self.files {
                match *file {
                    FuseFile::Dir(ref dir) => {
                        if name == dir.name {
                            reply.entry(&ttl, &dir.attr, 0);
                            return;
                        }
                    },
                    _ => {},
                }
            }
        } else {
            for (inode, file) in &self.files {
                if &parent == &file.parent() {
                    match *file {
                        FuseFile::File(ref file) => {
                            if name == "in" || name == "out" {
                                reply.entry(&ttl, &file.attr, 0);
                                return;
                            } else {
                                reply.error(ENOENT);
                                return;
                            }
                        },
                        _ => {},
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
                for (inode, file) in &self.files {
                    match *file {
                        FuseFile::Dir(ref dir) => {
                            let ino = dir.attr.ino;
                            reply.add(ino, ino, FileType::Directory, &dir.name);
                        },
                        _ => {},
                    }
                }
                reply.ok();
                return;
            } else {
                if let Some(file) = self.files.get(&ino) {
                    match *file {
                        FuseFile::Dir(ref dir) => {
                            reply.add(1, 0, FileType::Directory, ".");
                            reply.add(1, 1, FileType::Directory, "..");

                            let in_inode = dir.in_inode;
                            let out_inode = dir.out_inode;

                            reply.add(
                                in_inode,
                                in_inode,
                                FileType::RegularFile,
                                "in",
                            );

                            reply.add(
                                out_inode,
                                out_inode,
                                FileType::RegularFile,
                                "out",
                            );

                            reply.ok();
                            return;
                        },
                        _ => {},
                    }
                }
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

        if let Some(file) = self.files.get(&req_ino) {
            match *file {
                FuseFile::File(ref file) => {
                    reply.data(&file.buf[offset as usize..]);
                },
                _ => {},
            }
        } else {
            reply.error(ENOENT);
        }
    }
}
