extern crate libc;
use libc::{ENOENT, ENOTSUP};

extern crate fuse_mt;
use fuse_mt::*;

// extern crate irc;
// use irc::client::prelude::*;

extern crate time;
use time::Timespec;

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::collections::HashMap;

pub struct IrcFs {
    dirs: HashMap<OsString, IrcDir>,
    attr: FileAttr,
    in_file: IrcFile,
    out_file: IrcFile,
    highest_inode: u64,
}

struct IrcDir {
    // server: IrcServer,
    attr: FileAttr,
    in_file: IrcFile,
    out_file: IrcFile,
}

pub struct IrcFile {
    attr: FileAttr,
    buf: Vec<u8>,
}

impl IrcDir {
    fn new(ino: u64, in_inode: u64, out_inode: u64) -> Self {
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
            attr: attr,
            in_file: IrcFile::new(in_inode),
            out_file: IrcFile::new(out_inode),
        }
    }
}

impl IrcFile {
    fn new(ino: u64) -> Self {
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
    pub fn new(uid: u32, gid: u32) -> Self {
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
            uid: uid,
            gid: gid,
            rdev: 0,
            flags: 0,
        };

        IrcFs {
            dirs: HashMap::new(),
            attr: attr,
            in_file: IrcFile::new(2),
            out_file: IrcFile::new(3),
            highest_inode: 3,
        }
    }

    // pub fn add_server(&mut self, name: Option<OsString>, server: IrcServer) {
    pub fn add_server(&mut self, alias: OsString) {
        let dir_ino = self.highest_inode + 1;
        let in_ino = dir_ino + 1;
        let out_ino = dir_ino + 2;

        self.dirs.insert(alias, IrcDir::new(dir_ino, in_ino, out_ino));
        self.highest_inode += 3;
        self.attr.nlink += 1;
    }

    // pub fn entry(&self, path: &Path) -> Option<(OsString, FileType)> {
    //     if path == Path::new("/") {
    //         return Some(OsString::from("/"), FileType::Directory);
    //     } else if path == Path::new("/in") {
    //         return Some(OsString::from("in"), FileType::RegularFile);
    //     } else if path == Path::new("/out") {
    //         return Some(OsString::from("out"), FileType::RegularFile);
    //     }

    //     if let Some(parent) = path.parent() {
    //         if let Some(dir) = self.dirs.get(OsStr::new(parent)) {
    //             if path.file_name() == Some(OsStr::new("in")) {
    //                 return Some(OsString::from("in"), FileType::RegularFile);
    //             } else if path.file_name() == Some(OsStr::new("out")) {
    //                 return Some(OsString::from("out"), FileType::RegularFile);
    //             } else {
    //                 return Some(OsString::from(OsStr::new(parent)), FileType::Directory);
    //             }
    //         }
    //     }

    //     None
    // }

    pub fn attr(&self, path: &Path) -> Option<FileAttr> {
        if path == Path::new("/") {
            Some(self.attr)
        } else if path == Path::new("/in") {
            Some(self.in_file.attr)
        } else if path == Path::new("/out") {
            Some(self.out_file.attr)
        } else if let Some(dir) = self.dirs.get(path.iter().nth(1).unwrap()) {
            if path.file_name() == Some(OsStr::new("in")) {
                Some(dir.in_file.attr)
            } else if path.file_name() == Some(OsStr::new("out")) {
                Some(dir.out_file.attr)
            } else if path.iter().count() == 2 {
                Some(dir.attr)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl FilesystemMT for IrcFs {
    fn init(&mut self, _req: RequestInfo) -> ResultEmpty {
        let foo = OsString::from("foo");
        let bar = OsString::from("bar");
        let baz = OsString::from("baz");

        self.add_server(foo);
        self.add_server(bar);
        self.add_server(baz);

        self.out_file.insert_data("Hello, world\n".as_bytes());

        self.in_file.attr.uid = self.attr.uid;
        self.in_file.attr.gid = self.attr.gid;
        self.out_file.attr.uid = self.attr.uid;
        self.out_file.attr.gid = self.attr.gid;

        for dir in self.dirs.values_mut() {
            dir.attr.uid = self.attr.uid;
            dir.attr.gid = self.attr.gid;
            dir.in_file.attr.uid = self.attr.uid;
            dir.in_file.attr.gid = self.attr.gid;
            dir.out_file.attr.uid = self.attr.uid;
            dir.out_file.attr.gid = self.attr.gid;
        }

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

    fn getattr(&mut self, _req: RequestInfo, path: &Path, _fh: Option<u64>) -> ResultGetattr {
        if let Some(attr) = self.attr(&path) {
            Ok((Timespec::new(1, 0), attr))
        } else {
            Err(ENOENT)
        }
    }

    fn lookup(&mut self, _req: RequestInfo, parent: &Path, name: &OsStr) -> ResultEntry {
        if let Some(attr) = self.attr(&parent.join(name)) {
            Ok((Timespec::new(1, 0), attr))
        } else {
            Err(ENOENT)
        }
    }

    fn opendir(&mut self, _req: RequestInfo, path: &Path, _flags: u32) -> ResultOpen {
        if let Some(_attr) = self.attr(&path) {
            Ok((0, 0))
        } else {
            Err(ENOENT)
        }
    }

    fn readdir(&mut self, _req: RequestInfo, path: &Path, _fh: u64) -> ResultReaddir {
        if path == Path::new("/") {
            let mut entries = Vec::new();
            for (name, _dir) in &self.dirs {
                entries.push(DirectoryEntry { name: name.to_owned(), kind: FileType::Directory });
            }
            entries.push(DirectoryEntry {name:OsString::from("."),kind:FileType::Directory});
            entries.push(DirectoryEntry {name:OsString::from(".."),kind:FileType::Directory});
            entries.push(DirectoryEntry {name:OsString::from("in"),kind:FileType::RegularFile});
            entries.push(DirectoryEntry {name:OsString::from("out"),kind:FileType::RegularFile});
            Ok(entries)
        } else if let Some(name) = path.iter().nth(1) {
            let mut entries = Vec::new();
            if let Some(_dir) = self.dirs.get(name) {
                entries.push(DirectoryEntry {name:OsString::from("."),kind:FileType::Directory});
                entries.push(DirectoryEntry {name:OsString::from(".."),kind:FileType::Directory});
                entries.push(DirectoryEntry {name:OsString::from("in"),kind:FileType::RegularFile});
                entries.push(DirectoryEntry {name:OsString::from("out"),kind:FileType::RegularFile});
            }
            Ok(entries)
        } else {
            Err(ENOENT)
        }
    }

    // // fn setattr(&mut self,
    // //    _req: &Request,
    // //    ino: u64,
    // //    _mode: Option<u32>,
    // //    _uid: Option<u32>,
    // //    _gid: Option<u32>,
    // //    size: Option<u64>,
    // //    _atime: Option<Timespec>,
    // //    _mtime: Option<Timespec>,
    // //    _fh: Option<u64>,
    // //    _crtime: Option<Timespec>,
    // //    _chgtime: Option<Timespec>,
    // //    _bkuptime: Option<Timespec>,
    // //    _flags: Option<u32>,
    // //    reply: ReplyAttr) {
    // //     match self.types.get(&ino) {
    // //         Some(&FuseFiletype::Dir) => {
    // //         },
    // //         Some(&FuseFiletype::InFile) => {
    // //             if size == Some(0) {
    // //                 let ttl = Timespec::new(1, 0);
    // //                 reply.attr(&ttl, &self.files[&ino].attr);
    // //                 return;
    // //             }
    // //         },
    // //         Some(&FuseFiletype::OutFile) => {
    // //         },
    // //         None => {
    // //         },
    // //     }
    // //     reply.error(ENOTSUP);
    // // }

    // fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
    //     println!("opened(ino={}, flags={})", ino, flags);
    //     reply.opened(0, flags);
    // }

    fn read(&mut self,_req:RequestInfo,path:&Path,_fh:u64,offset:u64,_size:u32) -> ResultData {
        if path == Path::new("/in") {
            return Ok(self.in_file.buf[offset as usize..].to_owned());
        } else if path == Path::new("/out") {
            return Ok(self.out_file.buf[offset as usize..].to_owned());
        } else if let Some(name) = path.iter().nth(1) {
            if let Some(dir) = self.dirs.get(name) {
                if path.file_name() == Some(OsStr::new("in")) {
                    return Ok(dir.in_file.buf[offset as usize..].to_owned());
                } else if path.file_name() == Some(OsStr::new("out")) {
                    return Ok(dir.out_file.buf[offset as usize..].to_owned());
                }
            }
        }
        return Err(ENOENT);
    }

    fn truncate(&mut self,
                _req: RequestInfo,
                path: &Path,
                _fh: Option<u64>,
                _size: u64)
                -> ResultEmpty {
        if path == Path::new("/in") {
            return Ok(());
        } else if path == Path::new("/out") {
            return Err(ENOTSUP);
        } else if let Some(name) = path.iter().nth(1) {
            if let Some(_dir) = self.dirs.get_mut(name) {
                if path.file_name() == Some(OsStr::new("in")) {
                    return Ok(());
                } else if path.file_name() == Some(OsStr::new("out")) {
                    return Err(ENOTSUP);
                }
            }
        }
        return Err(ENOENT);
    }

    fn write(&mut self,
             _req: RequestInfo,
             path: &Path,
             _fh: u64,
             _offset: u64,
             data: &[u8],
             _flags: u32)
             -> ResultWrite {
        if path == Path::new("/in") {
            &self.in_file.insert_data(&data);
            return Ok(data.len() as u32);
        } else if path == Path::new("/out") {
            return Err(ENOTSUP);
        } else if let Some(name) = path.iter().nth(1) {
            if let Some(dir) = self.dirs.get_mut(name) {
                if path.file_name() == Some(OsStr::new("in")) {
                    &dir.in_file.insert_data(&data);
                    return Ok(data.len() as u32);
                } else if path.file_name() == Some(OsStr::new("out")) {
                    return Err(ENOTSUP);
                }
            }
        }
        return Err(ENOENT);
    }
}
