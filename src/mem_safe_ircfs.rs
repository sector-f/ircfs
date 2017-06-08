use fuse_mt::*;
use mem_safe_fs::*;
use config::FsConfig;
use libc::{c_int, ENOENT};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::path::Path;

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

        fs.mk_rw_file("/in", &req).unwrap();
        fs.mk_ro_file("/out", &req).unwrap();

        Ok(())
    }

    fn open(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpen {
        Ok((0, 0))
    }

    fn readdir(&self, req: RequestInfo, path: &Path, _fh: u64) -> ResultReaddir {
        let fs = self.fs.read().unwrap();
        match fs.dir_entries(&path, &req) {
            Ok(entries) => Ok(entries),
            Err(e) => Err(ENOENT),
        }
    }
}
