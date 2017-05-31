extern crate fuse;
use fuse::*;

extern crate libc;
use libc::{c_int, ENOENT, EISDIR, ENOTSUP, ENOSYS};

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

        let mut filesystem = IrcFs {
            root: IrcDir::new(1, uid, gid).into(),
            inodes: ino_map,
            highest_inode: 1,
        };

        let _ = filesystem.make_in_file("/in");
        let _ = filesystem.make_out_file("/out");
        filesystem
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
                let _ = self.make_in_file(path.join(Path::new("in")));
                let _ = self.make_out_file(path.join(Path::new("out")));
                return Ok(());
            },
            Err(_) => {
                return Err(());
            }
        }
    }

    pub fn make_in_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ()> {
        let path = path.as_ref();

        let ino = self.highest_inode + 1;
        let uid = self.root.attr().uid;
        let gid = self.root.attr().gid;

        let file = IrcFile::new_rw(ino, uid, gid);

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

    pub fn make_out_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ()> {
        let path = path.as_ref();

        let ino = self.highest_inode + 1;
        let uid = self.root.attr().uid;
        let gid = self.root.attr().gid;

        let file = IrcFile::new_ro(ino, uid, gid);

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

    pub fn get_mut_by_ino(&mut self, ino: u64) -> Option<&mut Node> {
        // TODO: get rid of to_owned() here
        let path = match self.inodes.get(&ino) {
            Some(path) => { path.to_owned() },
            None => { return None; },
        };

        self.get_mut_node(&path)
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

    fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        if let Some(_node) = self.get_by_ino(ino) {
            reply.opened(0, flags);
        } else {
            reply.error(ENOENT);
        }
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, _size: u32, reply: ReplyData) {
        match self.get_by_ino(ino) {
            Some(&Node::F(ref file)) => {
                reply.data(&file.data()[offset as usize..]);
            },
            Some(&Node::D(ref _dir)) => {
                reply.error(EISDIR);
            },
            None => {
                reply.error(ENOENT);
            },
        }
    }

    fn setattr(&mut self,
       _req: &Request,
       ino: u64,
       _mode: Option<u32>,
       _uid: Option<u32>,
       _gid: Option<u32>,
       size: Option<u64>,
       _atime: Option<Timespec>,
       _mtime: Option<Timespec>,
       _fh: Option<u64>,
       _crtime: Option<Timespec>,
       _chgtime: Option<Timespec>,
       _bkuptime: Option<Timespec>,
       _flags: Option<u32>,
        reply: ReplyAttr) {
        if let Some(_size) = size {
            match self.get_mut_by_ino(ino) {
                Some(&mut Node::F(ref mut file)) => {
                    reply.attr(&Timespec::new(1, 0), &file.attr());
                    return;
                },
                Some(&mut Node::D(ref mut dir)) => {
                    reply.attr(&Timespec::new(1, 0), &dir.attr());
                    return;
                },
                _ => {
                    reply.error(ENOENT);
                    return;
                },
            }
        }

        reply.error(ENOSYS);
    }

    fn write(&mut self, _req: &Request, ino: u64, _fh: u64, _offset: u64, data: &[u8], _flags: u32, reply: ReplyWrite) {
        match self.get_mut_by_ino(ino) {
            Some(&mut Node::F(ref mut file)) => {
                if file.is_readonly() {
                    reply.error(ENOTSUP);
                } else {
                    file.insert_data(&data);
                    reply.written(data.len() as u32);
                }
            },
            Some(&mut Node::D(ref mut _dir)) => {
                reply.error(EISDIR);
            },
            None => {
                reply.error(ENOENT);
            },
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, mut reply: ReplyDirectory) {
        if offset == 0 {
            let parent_ino = if ino == 1 {
                1
            } else {
                self.get_node(&self.inodes[&ino].parent().unwrap()).unwrap().attr().ino
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
