extern crate fuse;
use fuse::*;

extern crate libc;
use libc::{c_int, ENOENT, /*ENOTSUP*/};

extern crate time;
use time::Timespec;

use filesystem::*;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct IrcFs {
    root: Node,
    inodes: HashMap<u64, PathBuf>,
    highest_inode: u64,
}

impl IrcFs {
    pub fn new(uid: u32, gid: u32) -> Self {
        let mut ino_map = HashMap::new();
        ino_map.insert(1, PathBuf::from("/"));

        IrcFs {
            root: IrcDir::new(1, uid, gid).into(),
            inodes: ino_map,
            highest_inode: 1,
        }
    }

    pub fn make_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ()> {
        let path = path.as_ref();

        let ino = self.highest_inode + 1;
        let uid = self.root.attr().uid;
        let gid = self.root.attr().gid;

        let dir = IrcDir::new(ino, uid, gid);

        match self.insert_node(path, dir.into()) {
            Ok(_) => {
                self.highest_inode += 1;
                self.inodes.insert(self.highest_inode, path.to_owned());
                let _ = self.make_file(path.join(Path::new("in")));
                let _ = self.make_file(path.join(Path::new("out")));
                return Ok(());
            },
            Err(_) => {
                return Err(());
            }
        }
    }

    pub fn make_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ()> {
        let path = path.as_ref();

        let ino = self.highest_inode + 1;
        let uid = self.root.attr().uid;
        let gid = self.root.attr().gid;

        let file = IrcFile::new(ino, uid, gid);

        match self.insert_node(path, file.into()) {
            Ok(_) => {
                self.highest_inode += 1;
                self.inodes.insert(self.highest_inode, path.to_owned());
                return Ok(());
            },
            Err(_) => {
                return Err(());
            }
        }
    }

    pub fn insert_node<P: AsRef<Path>>(&mut self, path: P, node: Node) -> Result<(), ()> {
        let path = path.as_ref();
        let ino = node.attr().ino;

        if path.is_absolute() {
            let path = path.strip_prefix("/").unwrap();
            self.inodes.insert(ino, path.to_owned());
            self.root.as_mut_dir().insert_node(path, node)
        } else {
            Err(())
        }
    }

    pub fn get_by_ino(&self, ino: u64) -> Option<&Node> {
        self.inodes.get(&ino).and_then(|path| self.get_node(&path))
    }

    pub fn get_node<P: AsRef<Path>>(&self, path: P) -> Option<&Node> {
        let path = path.as_ref();

        if path.is_absolute() {
            if path == Path::new("/") {
                Some(&self.root)
            } else {
                let path = path.strip_prefix("/").unwrap();
                self.root.as_dir().get(path)
            }
        } else {
            None
        }
    }

    pub fn get_mut_node<P: AsRef<Path>>(&mut self, path: P) -> Option<&mut Node> {
        let path = path.as_ref();

        if path.is_absolute() {
            if path == Path::new("/") {
                Some(&mut self.root)
            } else {
                let path = path.strip_prefix("/").unwrap();
                self.root.as_mut_dir().get_mut(path)
            }
        } else {
            None
        }
    }
}

impl Filesystem for IrcFs {
    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        self.make_dir("/freenode").unwrap();
        self.make_dir("/freenode/##linux").unwrap();
        self.make_dir("/freenode/#bash").unwrap();

        self.make_dir("/rizon").unwrap();
        self.make_dir("/rizon/#code").unwrap();

        Ok(())
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if let Some(node) = self.get_by_ino(ino) {
            reply.attr(&Timespec::new(1, 0), &node.attr());
        } else {
            reply.error(ENOENT);
        }
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if let Some(&Node::D(ref dir)) = self.get_by_ino(parent) {
            if let Some(file) = dir.get(Path::new(name)) {
                reply.entry(&Timespec::new(1, 0), &file.attr(), 0);
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, mut reply: ReplyDirectory) {
        if offset == 0 {
            let parent_ino = if ino == 1 {
                1
            } else {
                self.get_node(&self.inodes[&ino]).unwrap().attr().ino
            };

            reply.add(
                ino,
                ino,
                FileType::Directory,
                Path::new("."),
            );

            reply.add(
                parent_ino,
                parent_ino,
                FileType::Directory,
                Path::new(".."),
            );

            if let Some(&Node::D(ref dir)) = self.get_by_ino(ino) {
                for (name, file) in &dir.map {
                    reply.add(
                        file.attr().ino,
                        file.attr().ino,
                        file.attr().kind,
                        &name
                    );
                }
            }
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }
}
