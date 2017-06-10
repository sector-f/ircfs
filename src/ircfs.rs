use config::FsConfig;
use libc::{ENOENT, EISDIR};
use std::sync::{Arc, RwLock};
use std::path::Path;
use time::Timespec;
use std::ffi::OsString;

use fuse_mt::*;
use filesystem::*;

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
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
        let mut fs = self.fs.write().unwrap();

        fs.mk_rw_file("/in").unwrap();
        fs.mk_ro_file("/out").unwrap();

        fs.mk_dir("/foo").unwrap();
        fs.mk_rw_file("/foo/in").unwrap();
        fs.mk_ro_file("/foo/out").unwrap();

        Ok(())
    }

    fn read(&self, _req:RequestInfo, path:&Path, _fh:u64, offset:u64, size:u32) -> ResultData {
        let fs = self.fs.read().unwrap();

        match fs.get(path) {
            Some(&Node::D(ref _dir)) => {
                Err(EISDIR)
            },
            Some(&Node::F(ref file)) => {
                Ok(file.data()[offset as usize..size as usize].to_owned())
            },
            None => {
                Err(ENOENT)
            },
        }
    }

    fn open(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpen {
        Ok((0, 0))
    }

    fn opendir(&self, _req: RequestInfo, path: &Path, _flags: u32) -> ResultOpen {
        let fs = self.fs.read().unwrap();

        if let Some(_node) = fs.get(path) {
            Ok((0, 0))
        } else {
            Err(ENOENT)
        }
    }

    fn getattr(&self, _req: RequestInfo, path: &Path, _fh: Option<u64>) -> ResultEntry {
        let fs = self.fs.read().unwrap();

        if let Some(node) = fs.get(path) {
            Ok((Timespec::new(1, 0), node.attr().clone()))
        } else {
            Err(ENOENT)
        }
    }

    fn readdir(&self, _req: RequestInfo, path: &Path, _fh: u64) -> ResultReaddir {
        let fs = self.fs.read().unwrap();

        match fs.dir_entries(&path) {
            Some(mut entries) => {
                entries.push(
                    DirectoryEntry {name: OsString::from("."), kind: FileType::Directory}
                );
                entries.push(
                    DirectoryEntry {name: OsString::from(".."), kind: FileType::Directory}
                );
                Ok(entries)
            },
            None => Err(ENOENT),
        }
    }

    // Allows directory traversal (I think)
    fn access(&self, _req: RequestInfo, _path: &Path, _mask: u32) -> ResultEmpty {
        Ok(())
    }
}
