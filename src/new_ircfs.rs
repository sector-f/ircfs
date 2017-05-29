extern crate fuse_mt;
use fuse_mt::*;

extern crate libc;
use libc::{ENOENT, ENOTSUP};

extern crate time;
use time::Timespec;

use filesystem::*;

use std::collections::HashMap;
use std::ffi::{OsString, OsStr};
use std::path::{self, Path, PathBuf};

#[derive(Debug)]
pub struct IrcFs {
    root: IrcDir,
    highest_inode: u64,
}

impl IrcFs {
    pub fn new(uid: u32, gid: u32) -> Self {
        IrcFs {
            root: IrcDir::new(1, uid, gid),
            highest_inode: 1,
        }
    }

    pub fn insert_node<P: AsRef<Path>>(&mut self, path: P, node: Node) -> Result<(), ()> {
        let path = path.as_ref();

        if path.is_absolute() {
            let path = path.strip_prefix("/").unwrap();
            self.root.insert_node(path, node)
        } else {
            Err(())
        }
    }

    pub fn get_node<P: AsRef<Path>>(&self, path: P) -> Option<&Node> {
        let path = path.as_ref();

        if path.is_absolute() {
            let path = path.strip_prefix("/").unwrap();
            self.root.get(path)
        } else {
            None
        }
    }

    pub fn get_mut_node<P: AsRef<Path>>(&mut self, path: P) -> Option<&mut Node> {
        let path = path.as_ref();

        if path.is_absolute() {
            let path = path.strip_prefix("/").unwrap();
            self.root.get_mut(path)
        } else {
            None
        }
    }

    pub fn next_inode(&mut self) -> u64 {
        self.highest_inode += 1;
        self.highest_inode
    }
}

impl FilesystemMT for IrcFs {
    fn init(&mut self, _req: RequestInfo) -> ResultEmpty {
        let uid = self.root.attr().uid;
        let gid = self.root.attr().gid;

        // Add servers
        let ino = self.next_inode();
        self.insert_node("/freenode", IrcDir::new(ino, uid, gid).into()).unwrap();
        let ino = self.next_inode();
        self.insert_node("/rizon", IrcDir::new(ino, uid, gid).into()).unwrap();

        let ino = self.next_inode();
        self.insert_node("/freenode/##linux", IrcDir::new(ino, uid, gid).into()).unwrap();
        let ino = self.next_inode();
        self.insert_node("/freenode/#bash", IrcDir::new(ino, uid, gid).into()).unwrap();
        let ino = self.next_inode();
        self.insert_node("/rizon/#code", IrcDir::new(ino, uid, gid).into()).unwrap();
        let ino = self.next_inode();
        self.insert_node("/rizon/#ircfs", IrcDir::new(ino, uid, gid).into()).unwrap();

        for node in self.root.map.values_mut() {
            if let &mut Node::D(ref mut server_dir) = node {
                for node in server_dir.map.values_mut() {
                    if let &mut Node::D(ref mut channel_dir) = node {
                        let ino = self.highest_inode + 1;
                        channel_dir.insert_node("in", IrcFile::new(ino, uid, gid).into()).unwrap();
                        let ino = self.highest_inode + 2;
                        channel_dir.insert_node("out", IrcFile::new(ino, uid, gid).into()).unwrap();
                        self.highest_inode += 2;
                    }
                }
            }
        }

        Ok(())
    }

    fn getattr(&mut self, _req: RequestInfo, path: &Path, _fh: Option<u64>) -> ResultGetattr {
        if path == Path::new("/") {
            Ok((Timespec::new(1, 0), self.root.attr()))
        } else if let Some(node) = self.get_node(&path) {
            Ok((Timespec::new(1, 0), node.attr()))
        } else {
            Err(ENOENT)
        }
    }

    fn lookup(&mut self, _req: RequestInfo, parent: &Path, name: &OsStr) -> ResultEntry {
        if name == OsStr::new("/") {
            Ok((Timespec::new(1, 0), self.root.attr()))
        } else if let Some(node) = self.get_node(&parent.join(name)) {
            Ok((Timespec::new(1, 0), node.attr()))
        } else {
            Err(ENOENT)
        }
    }

    fn opendir(&mut self, _req: RequestInfo, path: &Path, _flags: u32) -> ResultOpen {
        if path == Path::new("/") {
            Ok((0, 0))
        } else if let Some(_node) = self.get_node(&path) {
            Ok((0, 0))
        } else {
            Err(ENOENT)
        }
    }

    fn readdir(&mut self, _req: RequestInfo, path: &Path, _fh: u64) -> ResultReaddir {
        if path == Path::new("/") {
            let mut entries = Vec::new();
            for (name, file) in &self.root.map {
                entries.push(
                    DirectoryEntry {
                        name: name.to_owned(),
                        kind: file.attr().kind,
                    }
                );
            }
            return Ok(entries);
        } else if let Some(&Node::D(ref dir)) = self.get_node(path) {
            let mut entries = Vec::new();
            for (name, file) in &dir.map {
                entries.push(
                    DirectoryEntry {
                        name: name.to_owned(),
                        kind: file.attr().kind,
                    }
                );
            }
            return Ok(entries);
        } else {
            Err(ENOENT)
        }
    }
}
