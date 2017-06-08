use fuse_mt::*;
use mem_safe_fs::*;
// use tree_fs::*;
use config::FsConfig;
use libc::c_int;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

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

        fs.get("/", &req).unwrap();

        // fs.mk_rw_file("/in", &req).unwrap();
        // fs.mk_ro_file("/out", &req).unwrap();

        Ok(())
    }
}
