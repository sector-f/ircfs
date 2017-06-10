use config::FsConfig;
use libc::{c_int, ENOENT, EACCES};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::path::Path;
use time::Timespec;

use fuse_mt::*;
use filesystem::*;
use permissions::Mode;

pub struct IrcFs {
    fs: Arc<RwLock<Filesystem>>,
}

impl IrcFs {
    pub fn new(config: &FsConfig, uid: u32, gid: u32) -> Self {
        IrcFs {
            fs: Arc::new(RwLock::new(Filesystem::new(uid, gid))),
        }
    }
}

impl FilesystemMT for IrcFs {
    fn init(&self, req: RequestInfo) -> ResultEmpty {
        let mut fs = self.fs.write().unwrap();

        fs.mk_rw_file("/in").unwrap();
        fs.mk_ro_file("/out").unwrap();

        fs.mk_dir("/foo").unwrap();
        fs.mk_rw_file("/foo/in").unwrap();
        fs.mk_ro_file("/foo/out").unwrap();

        Ok(())
    }

    fn open(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpen {
        Ok((0, 0))
    }

    fn opendir(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpen {
        Ok((0, 0))
    }

    fn getattr(&self, req: RequestInfo, path: &Path, _fh: Option<u64>) -> ResultEntry {
        let fs = self.fs.read().unwrap();

        if let Some(node) = fs.get(path) {
            Ok((Timespec::new(1, 0), node.attr().clone()))
        } else {
            Err(ENOENT)
        }
    }

    fn readdir(&self, req: RequestInfo, path: &Path, _fh: u64) -> ResultReaddir {
        let fs = self.fs.read().unwrap();
        let dir = fs.get(&path).unwrap(); // This fn should only get valid paths. I hope.

        if can_read(&dir, &req) {
            match fs.dir_entries(&path) {
                Some(entries) => Ok(entries),
                None => Err(ENOENT),
            }
        } else {
            Err(EACCES)
        }
    }

    // Allows directory traversal (I think)
    fn access(&self, _req: RequestInfo, path: &Path, mask: u32) -> ResultEmpty {
        Ok(())
    }
}
