use fuse_mt::{FileAttr, FileType, RequestInfo, DirectoryEntry};

extern crate time;

use std::collections::HashMap;
use std::ffi::{OsString, OsStr};
use std::path::{PathBuf, Path};
use std::io::{self, Error, ErrorKind};

use permissions::*;

pub struct Filesystem {
    fake_root: FuseDir,
}

impl Filesystem {
    pub fn new(uid: u32, gid: u32) -> Self {
        let mut root = FuseDir::new(uid, gid);
        root.mk_dir("/", uid, gid).unwrap();

        Filesystem {
            fake_root: root,
        }
    }

    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<&Node> {
        self.fake_root.get(&path)
    }

    pub fn get_mut<P: AsRef<Path>>(&mut self, path: P) -> Option<&mut Node> {
        self.fake_root.get_mut(&path)
    }

    pub fn mk_dir<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let uid = self.root_dir().attr.uid;
        let gid = self.root_dir().attr.gid;
        self.fake_root.mk_dir(path, uid, gid)
    }

    pub fn mk_ro_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let uid = self.root_dir().attr.uid;
        let gid = self.root_dir().attr.gid;
        self.fake_root.mk_ro_file(path, uid, gid)
    }

    pub fn mk_rw_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let uid = self.root_dir().attr.uid;
        let gid = self.root_dir().attr.gid;
        self.fake_root.mk_rw_file(path, uid, gid)
    }

    pub fn dir_entries<P: AsRef<Path>>(&self, path: P)
    -> Option<Vec<DirectoryEntry>> {
        if let Some(&Node::D(ref dir)) = self.get(path) {
            let mut entries = Vec::new();
            for (name, node) in &dir.tree {
                entries.push(
                    DirectoryEntry {
                        name: name.to_owned(),
                        kind: node.attr().kind,
                    }
                );
            }
            Some(entries)
        } else {
            None
        }
    }

    fn root_dir(&self) -> &FuseDir {
        self.fake_root.get("/").unwrap().as_dir()
    }

    fn root_dir_mut(&mut self) -> &mut FuseDir {
        self.fake_root.get_mut("/").unwrap().as_mut_dir()
    }
}

pub struct FuseDir {
    tree: HashMap<OsString, Node>,
    attr: FileAttr,
}

impl FuseDir {
    fn new(uid: u32, gid: u32) -> Self {
        let init_time = time::get_time();

        FuseDir {
            tree: HashMap::new(),
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

    fn get<P: AsRef<Path>>(&self, path: P) -> Option<&Node> {
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
                Node::F(ref _file) => return None,
                Node::D(ref dir) => node = match dir.tree.get(segment) {
                    Some(node) => node,
                    None => return None,
                }
            }
        }

        Some(node)
    }

    fn get_mut<P: AsRef<Path>>(&mut self, path: P) -> Option<&mut Node> {
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
                Node::F(ref mut _file) => return None,
                Node::D(ref mut dir) => node = match dir.tree.get_mut(segment) {
                    Some(node) => node,
                    None => return None,
                }
            }
        }

        Some(node)
    }

    fn mk_dir<P: AsRef<Path>>(&mut self, path: P, uid: u32, gid: u32) -> io::Result<()> {
        self.insert_node(path, FuseDir::new(uid, gid).into())
    }

    fn mk_rw_file<P: AsRef<Path>>(&mut self, path: P, uid: u32, gid: u32) -> io::Result<()> {
        self.insert_node(path, FuseFile::new(uid, gid, true).into())
    }

    fn mk_ro_file<P: AsRef<Path>>(&mut self, path: P, uid: u32, gid: u32) -> io::Result<()> {
        self.insert_node(path, FuseFile::new(uid, gid, false).into())
    }

    fn insert_node<P: AsRef<Path>>(&mut self, path: P, node: Node) -> io::Result<()> {
        let path = path.as_ref();

        // Needed for making the root
        if path == Path::new("/") {
            if let Some(_node) = self.tree.get(OsStr::new(path)) {
                return Err(Error::from(ErrorKind::AlreadyExists));
            }
            self.tree.insert(OsString::from("/"), node);
            Ok(())
        } else {
            let parent = path.parent();
            let filename = path.file_name()
                .ok_or(Error::from(ErrorKind::InvalidInput,))?;

            if parent == Some(&Path::new("")) {
                if let Some(_n) = self.tree.get_mut(filename) {
                    return Err(Error::new(ErrorKind::AlreadyExists, "File already exists"));
                }
                self.tree.insert(filename.to_owned(), node);
                Ok(())
            } else {
                if let Some(segment) = parent {
                    match self.get_mut(segment) {
                        Some(&mut Node::D(ref mut dir)) => {
                            if let Some(_e) = dir.get(Path::new(filename)) {
                                return Err(Error::from(ErrorKind::AlreadyExists));
                            }
                            dir.tree.insert(filename.to_owned(), node);
                            Ok(())
                        },
                        Some(&mut Node::F(ref mut _file)) => {
                            Err(Error::from(ErrorKind::Other))
                        },
                        None => {
                            Err(Error::from(ErrorKind::NotFound))
                        },
                    }
                } else {
                    Err(Error::from(ErrorKind::NotFound))
                }
            }
        }
    }
}

pub struct FuseFile {
    attr: FileAttr,
    uw: bool,
    data: Vec<u8>,
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

pub enum Node {
    F(FuseFile),
    D(FuseDir),
}


impl Node {
    fn as_dir(&self) -> &FuseDir {
        match self {
            &Node::D(ref dir) => dir,
            &Node::F(ref _file) => panic!(),
        }
    }

    fn as_mut_dir(&mut self) -> &mut FuseDir {
        match self {
            &mut Node::D(ref mut dir) => dir,
            &mut Node::F(ref mut _file) => panic!(),
        }
    }

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

pub fn can_read(node: &Node, req: &RequestInfo) -> bool {
    let attr = node.attr();
    let mode = Mode::new(attr.perm).unwrap();

    if attr.uid == req.uid {
        mode.user.read
    } else if attr.gid == req.gid {
        mode.group.read
    } else {
        mode.other.read
    }
}

pub fn can_write(node: &Node, req: &RequestInfo) -> bool {
    let attr = node.attr();
    let mode = Mode::new(attr.perm).unwrap();

    if attr.uid == req.uid {
        mode.user.write
    } else if attr.gid == req.gid {
        mode.group.write
    } else {
        mode.other.write
    }
}

pub fn can_execute(node: &Node, req: &RequestInfo) -> bool {
    let attr = node.attr();
    let mode = Mode::new(attr.perm).unwrap();

    if attr.uid == req.uid {
        mode.user.execute
    } else if attr.gid == req.gid {
        mode.group.execute
    } else {
        mode.other.execute
    }
}
