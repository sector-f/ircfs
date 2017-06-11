use config::FsConfig;
use libc::{ENOENT, EISDIR, EACCES, ENOTSUP};
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

    // fn open(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpen {
    //     Ok((0, 0))
    // }

    fn read(&self, _req:RequestInfo, path:&Path, _fh:u64, offset:u64, size:u32) -> ResultData {
        let fs = self.fs.read().unwrap();

        match fs.get(path) {
            Some(&Node::D(ref _dir)) => {
                Err(EISDIR)
            },
            Some(&Node::F(ref file)) => {
                let data = file.data();
                let end = {
                    if (size as u64 + offset) as usize > data.len() {
                        data.len()
                    } else {
                        size as usize
                    }
                };
                Ok(data[offset as usize..end].to_owned())
            },
            None => {
                Err(ENOENT)
            },
        }
    }

    fn write(&self, req: RequestInfo, path: &Path, _fh: u64, _offset: u64, data: Vec<u8>, _flags: u32) -> ResultWrite {
        let mut fs = self.fs.write().unwrap();

        match fs.get_mut(path) {
            Some(&mut Node::D(ref mut _dir)) => {
                Err(EISDIR)
            },
            Some(&mut Node::F(ref mut file)) => {
                let uid = file.attr.uid;
                let gid = file.attr.gid;
                let mode = file.attr.perm;

                if can_write(uid, gid, mode, &req) {
                    file.insert_data(&data);
                    Ok(data.len() as u32)
                } else {
                    // Should probably be changed to EACCES if/when permissions are implemented
                    // But, currently, this will just be the "out" files, and ENOTSUP seems
                    // more logical
                    Err(ENOTSUP)
                }
            },
            None => {
                Err(ENOENT)
            },
        }
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
