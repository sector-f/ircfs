use libc::{ENOENT, EISDIR, ENOTSUP};
use time::{self, Timespec};
use irc::client::prelude::*;

use std::sync::{Arc, RwLock, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::path::{Path, PathBuf};
use std::ffi::OsString;
use std::collections::HashMap;
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration;
use std::io;

use fuse_mt::*;
use filesystem::*;

pub struct IrcFs {
    fs: Arc<RwLock<Filesystem>>,
    config: Config,
    tx_to_server: Mutex<Sender<Message>>,
}

impl IrcFs {
    pub fn new(config: &Config, uid: u32, gid: u32) -> io::Result<Self> {
        let (tx_to_fs, rx_from_server) = channel();
        let tx_to_server = init_server(config.clone(), tx_to_fs.clone())?;

        let filesystem = IrcFs {
            fs: Arc::new(RwLock::new(Filesystem::new(uid, gid))),
            config: config.clone(),
            tx_to_server: Mutex::new(tx_to_server),
        };

        if let Some(ref channels) = config.channels {
            let mut fs = filesystem.fs.clone();
            let mut fs = fs.write().unwrap();
            for channel in channels {
                let path = Path::new("/").join(channel);
                fs.mk_parents(&path);
                fs.mk_ro_file(&path.join("out"));
                fs.mk_rw_file(&path.join("in"));
            }
        }

        let fs = filesystem.fs.clone();
        thread::spawn(move || {
            for message in rx_from_server.iter() {
                let mut fs = fs.write().unwrap();
                match message {
                    FsControl::Message(ref path, ref data) => {
                        if let Some(&mut Node::F(ref mut file)) = fs.get_mut(path) {
                            file.insert_data(&data);
                        }
                    },
                    FsControl::CreateDir(ref path) => {
                        fs.mk_parents(&path);
                        fs.mk_ro_file(&path.join("out"));
                        fs.mk_rw_file(&path.join("in"));
                    }
                }
            }
        });

        return Ok(filesystem);
    }
}

fn init_server(config: Config, tx_to_fs: Sender<FsControl>) -> io::Result<Sender<Message>> {
    let (tx, rx) = channel::<Message>();

    let srv = IrcServer::from_config(config.clone())?;

    // This thread iterates over messages from the server
    // and sends the necessary actions to the filesystem, e.g. writing to files
    let server = srv.clone();
    thread::spawn(move || {
        server.identify();
        let root = Path::new("/");
        for msg_res in server.iter() {
            if let Ok(msg) = msg_res {
                match msg.command {
                    Command::PRIVMSG(target, message) => {
                        let time = time::now();
                        let username = msg.prefix.unwrap();
                        let username = username.split('!').nth(0).unwrap();
                        let chan_path = if &target == server.current_nickname() {
                            root.join(username)
                        } else {
                            root.join(target)
                        };
                        tx_to_fs.send(FsControl::CreateDir(chan_path.clone()));
                        tx_to_fs.send(
                            FsControl::Message(
                                chan_path.clone().join("out"),
                                format!("{} {}: {}\n",
                                    time.strftime("%T").unwrap(),
                                    username,
                                    message,
                                ).into_bytes(),
                            )
                        );
                    },
                    _ => {},
                }
            }
        }
    });

    // This thread receives messages from the filesystem and performs
    // the necessary actions, such as sending a PRIVMSG to the server
    let server = srv.clone();
    thread::spawn(move || {
        for message in rx.iter() {
            server.send(message);
        }
    });

    Ok(tx)
}

impl FilesystemMT for IrcFs {
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
        let mut fs = self.fs.write().unwrap();

        fs.mk_rw_file("/in").unwrap();
        fs.mk_ro_file("/out").unwrap();

        Ok(())
    }

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

    fn truncate(&self,req:RequestInfo,path:&Path,_fh:Option<u64>,_size:u64) -> ResultEmpty {
        let fs = self.fs.read().unwrap();

        match fs.get(path) {
            Some(&Node::D(ref _dir)) => {
                Err(EISDIR)
            },
            Some(&Node::F(ref file)) => {
                let uid = file.attr.uid;
                let gid = file.attr.gid;
                let mode = file.attr.perm;

                if can_write(uid, gid, mode, &req) {
                    Ok(())
                } else {
                    Err(ENOTSUP)
                }
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
}

#[derive(Debug, Clone)]
enum FsControl {
    CreateDir(PathBuf),
    Message(PathBuf, Vec<u8>),
}
