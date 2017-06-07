use fuse_mt::*;
use mem_safe_fs::*;
// use tree_fs::*;
use config::FsConfig;
use std::sync::RwLock;
use libc::c_int;
use std::ops::Deref;

pub struct IrcFs {
    root: Filesystem,
}

impl IrcFs {
    pub fn new(config: &FsConfig, uid: u32, gid: u32) -> Self {
        unimplemented!();
    }
}

impl FilesystemMT for IrcFs {
    fn init(&self, req: RequestInfo) -> ResultEmpty {
        // let lock = self.root.get("/in").unwrap();
        // let ref mut in_file = *lock.write().unwrap();

        // match *in_file {
        //     Node::F(ref mut file) => {
        //         file.insert_data("Testing\n".as_bytes());
        //     },
        //     Node::D(ref mut dir) => {
        //     },
        // }

        Ok(())
    }
}
