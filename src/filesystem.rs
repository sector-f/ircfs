use fuse_mt;
use fuse_mt::{FileAttr, FileType};

extern crate time;
use time::get_time;

use std::collections::HashMap;
use std::ffi::{OsString, OsStr};
use std::path::{self, Path};

#[derive(Debug)]
pub struct IrcDir {
    pub map: HashMap<OsString, Node>,
    attr: FileAttr,
}

#[derive(Debug)]
pub enum Node {
    F(IrcFile),
    D(IrcDir),
}

#[derive(Debug)]
pub struct IrcFile {
    attr: FileAttr,
    data: Vec<u8>,
}

impl IrcDir {
    pub fn new(ino: u64, uid: u32, gid: u32) -> Self {
        let init_time = time::get_time();

        let attr = FileAttr {
            ino: ino,
            size: 4096,
            blocks: 8,
            atime: init_time,
            mtime: init_time,
            ctime: init_time,
            crtime: init_time,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: uid,
            gid: gid,
            rdev: 0,
            flags: 0,
        };

        IrcDir {
            map: HashMap::new(),
            attr: attr,
        }
    }

    pub fn get(&self, path: &Path) -> Option<&Node> {
        let mut iter = path.iter();
        let first_segment = match iter.next() {
            Some(segment) => segment,
            None => return None,
        };
        let mut node = match self.map.get(first_segment) {
            Some(node) => node,
            None => return None,
        };

        for segment in iter {
            match *node {
                Node::F(ref _file) => return None,
                Node::D(ref dir) => node = match dir.map.get(segment) {
                    Some(node) => node,
                    None => return None,
                }
            }
        }

        Some(node)
    }

    pub fn get_mut(&mut self, path: &Path) -> Option<&mut Node> {
        let mut iter = path.iter();
        let first_segment = match iter.next() {
            Some(segment) => segment,
            None => return None,
        };
        let mut node = match self.map.get_mut(first_segment) {
            Some(node) => node,
            None => return None,
        };

        for segment in iter {
            match *{node} { // the trick is the {}
                Node::F(ref mut _file) => return None,
                Node::D(ref mut dir) => node = match dir.map.get_mut(segment) {
                    Some(node) => node,
                    None => return None,
                }
            }
        }

        Some(node)
    }

    pub fn insert_node<P: AsRef<Path>>(&mut self, path: P, node: Node) -> Result<(),()> {
        let path = path.as_ref();

        let parent = path.parent();
        let filename = path.file_name().ok_or(())?;

        if parent == Some(&Path::new("")) {
            if let Some(_n) = self.map.get_mut(filename) {
                return Err(());
            }
            self.map.insert(filename.to_owned(), node);
            return Ok(());
        } else {
            if let Some(segment) = parent {
                if let Some(&mut Node::D(ref mut dir)) = self.get_mut(segment) {
                    if let None = dir.get(Path::new(filename)) {
                        if let Node::D(ref _dir) = node {
                            dir.attr.nlink += 1;
                        }
                        dir.map.insert(filename.to_owned(), node);
                        return Ok(());
                    }
                }
            }
        }

        Err(())
    }

    pub fn attr(&self) -> FileAttr {
        self.attr
    }
}

impl Node {
    pub fn attr(&self) -> FileAttr {
        match *self {
            Node::F(ref file) => file.attr,
            Node::D(ref dir) => dir.attr,
        }
    }
}

impl From<IrcFile> for Node {
    fn from(f: IrcFile) -> Node {
        Node::F(f)
    }
}

impl From<IrcDir> for Node {
    fn from(d: IrcDir) -> Node {
        Node::D(d)
    }
}

// enum DirType {
//     Root,
//     Server,
//     Channel,
// }

impl IrcFile {
    pub fn new(ino: u64, uid: u32, gid: u32) -> Self {
        let init_time = time::get_time();

        let attr = FileAttr {
            ino: ino,
            size: 0,
            blocks: 1,
            atime: init_time,
            mtime: init_time,
            ctime: init_time,
            crtime: init_time,
            kind: FileType::RegularFile,
            perm: 0o644,
            nlink: 1,
            uid: uid,
            gid: gid,
            rdev: 0,
            flags: 0,
        };

        IrcFile {
            attr: attr,
            data: Vec::new(),
        }
    }

    pub fn insert_data(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
        self.attr.size += data.len() as u64;
    }

    pub fn attr(&self) -> FileAttr {
        self.attr
    }
}

