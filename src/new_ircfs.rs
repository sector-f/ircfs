extern crate fuse_mt;
use fuse_mt::*;

extern crate time;
use time::Timespec;

use filesystem::*;

use std::collections::HashMap;
use std::ffi::{OsString, OsStr};
use std::path::{self, Path, PathBuf};

pub struct IrcFs {
    root: IrcDir,
}

impl IrcFs {
    pub fn new(uid: u32, gid: u32) -> Self {
        IrcFs {
            root: IrcDir::new(1, uid, gid),
        }
    }

    // pub fn new_mock() -> Self {
    //     let init_time = time::get_time();

    //     let attr = FileAttr {
    //         ino: 1,
    //         size: 4096,
    //         blocks: 8,
    //         atime: init_time,
    //         mtime: init_time,
    //         ctime: init_time,
    //         crtime: init_time,
    //         kind: FileType::Directory,
    //         perm: 0o755,
    //         nlink: 2, // Number of hard links?
    //         uid: 0,
    //         gid: 0,
    //         rdev: 0,
    //         flags: 0,
    //     };

    //     IrcFs {
    //         root: IrcDir::new(1, 0, 0),
    //     }
    // }
}

impl FilesystemMT for IrcFs {

}
