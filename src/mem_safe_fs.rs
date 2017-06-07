use fuse_mt::{FileAttr, FileType};

extern crate time;

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{PathBuf, Path};
use std::sync::{Arc, RwLock, Mutex, RwLockReadGuard};
use std::io::{self, Error, ErrorKind};

////////////////
// Filesystem //
////////////////

fn new_lock<T>(item: T) -> Arc<RwLock<T>> {
    Arc::new(RwLock::new(item))
}

pub struct Filesystem {
    dir_tree: Arc<RwLock<LookupDir>>,
    files: Arc<RwLock<HashMap<usize, Arc<RwLock<Node>>>>>,
    highest_fh: Arc<RwLock<usize>>,
}

impl Filesystem {
    pub fn new(uid: u32, gid: u32) -> Self {
        let mut root = LookupDir::new(0);
        root.mk_dir("/", 1);

        let mut files = HashMap::new();
        files.insert(1, new_lock(FuseDir::new(uid, gid).into()));

        Filesystem {
            dir_tree: new_lock(root),
            files: new_lock(files),
            highest_fh: new_lock(1),
        }
    }

    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<&Arc<RwLock<Node>>> {
        let path = path.as_ref();

        if path.is_relative() {
            return None;
        }

        let dir_tree = &*self.dir_tree.read().unwrap();

        let fh = match dir_tree.get(path.strip_prefix("/").unwrap()) {
            Some(node) => node.fh(),
            None => return None,
        };

        let files = self.files.clone();
        files.read().unwrap().get(&fh)

        // self.files.read().unwrap().get(&fh).clone()
    }

    pub fn mk_dir<P: AsRef<Path>>(&self, path: P, fh: usize) -> io::Result<()> {
        // let path = path.as_ref();

        // if path.is_relative() {
        //     return Err(Error::new(ErrorKind::NotFound, "Given path must be absolute"));
        // }

        // let dir_tree = self.dir_tree.clone();

        // match dir_tree.write().unwrap().mk_dir(path, fh) {
        //     Ok(_) => {
        //         *self.highest_fh.write().unwrap() += 1;

        //         let uid = unimplemented!();
        //         let gid = unimplemented!();

        //         let fh = self.highest_fh.clone();

        //         self.files.insert(*fh.read().unwrap(), Arc::new(RwLock::new(FuseDir::new(uid, gid).into())));

        //         return Ok(());
        //     },
        //     Err(e) => {
        //         return Err(e);
        //     },
        // }

        unreachable!();
    }

    pub fn mk_ro_file<P: AsRef<Path>>(&self, path: P, fh: usize) -> io::Result<()> {
        unimplemented!();
    }

    pub fn mk_rw_file<P: AsRef<Path>>(&self, path: P, fh: usize) -> io::Result<()> {
        unimplemented!();
    }
}

////////////
// Lookup //
////////////

pub struct LookupDir {
    tree: HashMap<OsString, LookupNode>,
    fh: usize,
}

impl LookupDir {
    fn new(fh: usize) -> Self {
        LookupDir {
            tree: HashMap::new(),
            fh: fh,
        }
    }

    fn get<P: AsRef<Path>>(&self, path: P) -> Option<&LookupNode> {
        let path = path.as_ref();

        let mut iter = path.iter();
        let first_segment = match iter.next() {
            Some(segment) => segment,
            None => return None,
        };

        let mut node = match self.tree.get(first_segment) {
            Some(node) => node,
            None => return None,
        };

        for segment in iter {
            match *node {
                LookupNode::F(ref _file) => return None,
                LookupNode::D(ref dir) => node = match dir.tree.get(segment) {
                    Some(node) => node,
                    None => return None,
                }
            }
        }

        Some(node)
    }

    fn get_mut<P: AsRef<Path>>(&mut self, path: P) -> Option<&mut LookupNode> {
        let path = path.as_ref();

        let mut iter = path.iter();
        let first_segment = match iter.next() {
            Some(segment) => segment,
            None => return None,
        };

        let mut node = match self.tree.get_mut(first_segment) {
            Some(node) => node,
            None => return None,
        };

        for segment in iter {
            match *{node} {
                LookupNode::F(ref mut _file) => return None,
                LookupNode::D(ref mut dir) => node = match dir.tree.get_mut(segment) {
                    Some(node) => node,
                    None => return None,
                }
            }
        }

        Some(node)
    }

    fn mk_dir<P: AsRef<Path>>(&mut self, path: P, fh: usize) -> io::Result<()> {
        self.insert_node(path, LookupDir::new(fh).into())
    }

    fn mk_file<P: AsRef<Path>>(&mut self, path: P, fh: usize) -> io::Result<()> {
        self.insert_node(path, LookupFile::new(fh).into())
    }

    fn insert_node<P: AsRef<Path>>(&mut self, path: P, node: LookupNode) -> io::Result<()> {
        let path = path.as_ref();

        let parent = path.parent();
        let filename = path.file_name()
            .ok_or(Error::new(ErrorKind::InvalidInput, "No filename specified"))?;

        if parent == Some(&Path::new("")) {
            if let Some(_n) = self.tree.get_mut(filename) {
                return Err(Error::new(ErrorKind::AlreadyExists, "File already exists"));
            }
            self.tree.insert(filename.to_owned(), node);
            Ok(())
        } else {
            if let Some(segment) = parent {
                match self.get_mut(segment) {
                    Some(&mut LookupNode::D(ref mut dir)) => {
                        if let None = dir.get(Path::new(filename)) {
                            dir.tree.insert(filename.to_owned(), node);
                            Ok(())
                        } else {
                            Err(Error::new(ErrorKind::AlreadyExists, "File already exists"))
                        }
                    },
                    Some(&mut LookupNode::F(ref mut _file)) => {
                        Err(Error::new(ErrorKind::Other, "Not a directory"))
                    },
                    None => {
                        Err(Error::new(ErrorKind::NotFound, "No such file or directory"))
                    },
                }
            } else {
                Err(Error::new(ErrorKind::NotFound, "No such file or directory"))
            }
        }
    }
}

pub struct LookupFile {
    fh: usize,
}

impl LookupFile {
    fn new(fh: usize) -> Self {
        LookupFile {
            fh: fh,
        }
    }
}

pub enum LookupNode {
    F(LookupFile),
    D(LookupDir),
}

impl LookupNode {
    fn fh(&self) -> usize {
        match *self {
            LookupNode::F(ref file) => file.fh,
            LookupNode::D(ref dir) => dir.fh,
        }
    }
}

impl From<LookupFile> for LookupNode {
    fn from(f: LookupFile) -> LookupNode {
        LookupNode::F(f)
    }
}

impl From<LookupDir> for LookupNode {
    fn from(d: LookupDir) -> LookupNode {
        LookupNode::D(d)
    }
}

//////////////////
// Actual files //
//////////////////

pub struct FuseDir {
    attr: FileAttr,
}

pub struct FuseFile {
    attr: FileAttr,
    uw: bool, // user-writable
    data: Vec<u8>,
}

pub enum Node {
    F(FuseFile),
    D(FuseDir),
}

impl FuseDir {
    pub fn new(uid: u32, gid: u32) -> Self {
        let init_time = time::get_time();

        FuseDir {
            attr: FileAttr {
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
            }
        }
    }
}

impl FuseFile {
    pub fn new(uid: u32, gid: u32, uw: bool) -> Self {
        let init_time = time::get_time();

        let attr = FileAttr {
            size: 0,
            blocks: 1,
            atime: init_time,
            mtime: init_time,
            ctime: init_time,
            crtime: init_time,
            kind: FileType::RegularFile,
            perm: 0o600,
            nlink: 1,
            uid: uid,
            gid: gid,
            rdev: 0,
            flags: 0,
        };

        FuseFile {
            attr: attr,
            uw: uw,
            data: Vec::new(),
        }
    }

    pub fn insert_data(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
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
