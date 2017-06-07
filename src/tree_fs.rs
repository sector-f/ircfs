use fuse_mt::{FileAttr, FileType};

extern crate time;

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{PathBuf, Path};
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard};

pub struct Filesystem {
    files: HashMap<OsString, Arc<RwLock<Node>>>,
    root_attr: FileAttr,
}

pub enum Node {
    F(FuseFile),
    D(FuseDir),
}

pub struct FuseDir {
    files: HashMap<OsString, Arc<RwLock<Node>>>,
    attr: FileAttr,
}

pub struct FuseFile {
    attr: FileAttr,
    ro: bool,
    data: Vec<u8>,
}

impl Filesystem {
    pub fn new(uid: u32, gid: u32) -> Self {
        let init_time = time::get_time();

        let attr = FileAttr {
            size: 4096,
            blocks: 8,
            atime: init_time,
            mtime: init_time,
            ctime: init_time,
            crtime: init_time,
            kind: FileType::Directory,
            perm: 0o700,
            nlink: 2,
            uid: uid,
            gid: gid,
            rdev: 0,
            flags: 0,
        };

        Filesystem {
            files: HashMap::new(),
            root_attr: attr,
        }
    }

    pub fn mk_dir<P: AsRef<Path>>(&self, path: P) -> Result<(), ()> {
        // get uid/gid of parent dir
        let _path = path.as_ref();
        unimplemented!();
    }

    pub fn mk_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ()> {
        // get uid/gid of parent dir
        let _path = path.as_ref();
        unimplemented!();
    }

    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<RwLockReadGuard<Node>> {
        let path = path.as_ref();

        let mut iter = path.iter();
        let first_segment = match iter.next() {
            Some(segment) => segment,
            None => return None,
        };
        let mut node = match self.files.get(first_segment) {
            Some(node) => node.read(),
            None => return None,
        };


        for segment in iter {
            match node {
                Node::F(ref _file) => return None,
                Node::D(ref dir) => node = match dir.files.get(segment) {
                    Some(node) => node.read(),
                    None => return None,
                }
            }
        }

        Some(node)
    }
}

impl FuseDir {
    pub fn new(uid: u32, gid: u32) -> Self {
        let init_time = time::get_time();

        let attr = FileAttr {
            size: 4096,
            blocks: 8,
            atime: init_time,
            mtime: init_time,
            ctime: init_time,
            crtime: init_time,
            kind: FileType::Directory,
            perm: 0o700,
            nlink: 2,
            uid: uid,
            gid: gid,
            rdev: 0,
            flags: 0,
        };

        FuseDir {
            files: HashMap::new(),
            attr: attr,
        }
    }

    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<&Node> {
        unimplemented!();
    }
}

impl FuseFile {
    pub fn new() -> Self {
        unimplemented!();
    }
}

impl Node {
    pub fn attr(&self) -> &FileAttr {
        match *self {
            Node::F(ref file) => &file.attr,
            Node::D(ref dir) => &dir.attr,
        }
    }
}

impl From<FuseFile> for Node {
    fn from(f: FuseFile) -> Node {
        Node::F(f)
    }
}

impl From<FuseDir> for Node {
    fn from(d: FuseDir) -> Node {
        Node::D(d)
    }
}
