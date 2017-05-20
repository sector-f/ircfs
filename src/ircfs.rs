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
    dirs: HashMap<u64, IrcDir>, // Maps inodes to directories
    files: HashMap<u64, IrcFile>, // Maps inodes to in/out files (including fs root's)
    types: HashMap<u64, FuseFiletype>,
    root: RootDir,
    highest_inode: u64,
}

enum FuseFiletype {
    Dir,
    InFile,
    OutFile,
}

struct RootDir {
    attr: FileAttr,
    in_inode: u64,
    out_inode: u64,
}

struct IrcDir {
    name: OsString,
    // server: IrcServer,
    parent: u64,
    attr: FileAttr,
    in_inode: u64,
    out_inode: u64,
}

pub struct IrcFile {
    parent: u64,
    attr: FileAttr,
    buf: Vec<u8>,
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

        let mut files = HashMap::new();
        let infile = IrcFile::new(2, 1);
        let outfile = IrcFile::new(3, 1);

        files.insert(2, infile);
        files.insert(3, outfile);

        let mut types = HashMap::new();
        types.insert(2, FuseFiletype::InFile);
        types.insert(3, FuseFiletype::OutFile);

        IrcFs {
            dirs: HashMap::new(),
            files: files,
            types: types,
            root: RootDir { attr: attr, in_inode: 2, out_inode: 3 },
            highest_inode: 3,
        }
    }

    // pub fn add_server(&mut self, name: Option<OsString>, server: IrcServer) {
    pub fn add_server(&mut self, name: OsString) {
        let dir_ino = self.highest_inode + 1;
        let in_ino = dir_ino + 1;
        let out_ino = dir_ino + 2;

        let infile = IrcFile::new(in_ino, dir_ino);
        let mut outfile = IrcFile::new(out_ino, dir_ino);
        outfile.insert_data("Test data\n".as_bytes());
        let dir = IrcDir::new(name, dir_ino, in_ino, out_ino);

        self.dirs.insert(dir_ino, dir);
        self.types.insert(dir_ino, FuseFiletype::Dir);

        self.files.insert(in_ino, infile);
        self.types.insert(in_ino, FuseFiletype::InFile);

        self.files.insert(out_ino, outfile);
        self.types.insert(out_ino, FuseFiletype::OutFile);

        self.highest_inode += 3;
        self.root.attr.nlink += 1;
    }

    pub fn attr(&self, ino: u64) -> Option<FileAttr> {
        if ino == 1 {
            Some(self.root.attr)
        } else {
            match self.types.get(&ino) {
                Some(&FuseFiletype::Dir) => {
                    Some(self.dirs[&ino].attr)
                },
                Some(&FuseFiletype::InFile) => {
                    Some(self.files[&ino].attr)
                },
                Some(&FuseFiletype::OutFile) => {
                    Some(self.files[&ino].attr)
                },
                None => {
                    None
                },
            }
        }
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

        self.root.attr.uid = req.uid();
        self.root.attr.gid = req.gid();

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
        if let Some(attr) = self.attr(req_ino) {
            let ttl = Timespec::new(1, 0);
            reply.attr(&ttl, &attr);
            return;
        }
        reply.error(ENOENT);
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let ttl = Timespec::new(1, 0);
        if parent == 1 {
            if name == "in" {
                reply.entry(&ttl, &self.files[&self.root.in_inode].attr, 0);
                return;
            } else if name == "out" {
                reply.entry(&ttl, &self.files[&self.root.out_inode].attr, 0);
                return;
            } else {
                for (inode, dir) in &self.dirs {
                    if name == dir.name {
                        reply.entry(&ttl, &dir.attr, 0);
                        return;
                    }
                }
            }
        } else {
            if let Some(dir) = self.dirs.get(&parent) {
                if name == "in" {
                    reply.entry(&ttl, &self.files[&dir.in_inode].attr, 0);
                    return;
                } else if name == "out" {
                    reply.entry(&ttl, &self.files[&dir.out_inode].attr, 0);
                    return;
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
                reply.add(2, 2, FileType::RegularFile, "in");
                reply.add(3, 3, FileType::RegularFile, "out");

                for (inode, dir) in &self.dirs {
                    let inode = inode.clone();
                    reply.add(inode, inode, FileType::Directory, &dir.name);
                }

                reply.ok();
                return;
            } else {
                if let Some(dir) = self.dirs.get(&ino) {
                    // reply.add(1, 0, FileType::Directory, ".");
                    // reply.add(1, 1, FileType::Directory, "..");
                    reply.add(dir.in_inode, dir.in_inode, FileType::RegularFile, "in");
                    reply.add(dir.out_inode, dir.out_inode, FileType::RegularFile, "out");
                    reply.ok();
                    return;
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
            reply.data(&file.buf[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
    }
}
