extern crate fuse_mt;
use fuse_mt::*;

extern crate time;
use time::Timespec;

use std::collections::HashMap;
use std::ffi::OsString;

pub struct IrcFs {
    root: IrcDir,
}

struct IrcDir {
    map: HashMap<OsString, Node>,
    attr: FileAttr,
}

enum Node {
    F(IrcFile),
    D(IrcDir),
}

struct IrcFile {
    buf: Vec<u8>,
    attr: FileAttr,
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
            root: IrcDir {
                map: HashMap::new(),
                attr: attr,
            }
        }
    }
}

impl FilesystemMT for IrcFs {

}
