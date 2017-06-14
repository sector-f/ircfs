use libc::{ENOENT, EISDIR, ENOTSUP};
use time::Timespec;
use irc::client::prelude::*;

use std::sync::{Arc, RwLock, Mutex, mpsc};
use std::path::{Path, PathBuf};
use std::ffi::OsString;
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration;

use config::{FsConfig, convert_config};
use fuse_mt::*;
use filesystem::*;

pub struct IrcFs {
    fs: Arc<RwLock<Filesystem>>,
    config: FsConfig,
    tx: Mutex<mpsc::Sender<ControlCommand>>,
}

impl IrcFs {
    pub fn new(config: &FsConfig, uid: u32, gid: u32) -> Self {
        let (tx, rx) = mpsc::channel();

        let filesystem = IrcFs {
            fs: Arc::new(RwLock::new(Filesystem::new(uid, gid))),
            config: config.clone(),
            tx: Mutex::new(tx.clone()),
        };

        let fs = filesystem.fs.clone();
        thread::spawn(move || {
            for message in rx.iter() {
                let mut fs = fs.write().unwrap();
                match message {
                    ControlCommand::Message(ref path, ref data) => {
                        if let Some(&mut Node::F(ref mut file)) = fs.get_mut(path) {
                            file.insert_data(&data);
                        }
                    },
                    ControlCommand::CreateDir(ref path) => {
                        fs.mk_parents(&path);
                        fs.mk_ro_file(&path.join("out"));
                        fs.mk_rw_file(&path.join("in"));
                    }
                }
            }
        });

        return filesystem;
    }

    fn handle_server(&self, srv_conf: Config) {
        let tx = self.tx.lock().unwrap();
        let tx = tx.clone();

        thread::spawn(move || {
            if let Ok(server) = IrcServer::from_config(srv_conf.clone()) {
                server.identify();
                let root = Path::new("/");
                let server_path = root.join(srv_conf.server.unwrap());

                tx.send(ControlCommand::CreateDir(server_path.clone()));

                for msg_res in server.iter() {
                    if let Ok(msg) = msg_res {
                        match msg.command {
                            Command::PRIVMSG(target, message) => {
                                let username = msg.prefix.unwrap();
                                let username = username.split('!').nth(0).unwrap();
                                let chan_path = server_path.join(target);
                                tx.send(ControlCommand::CreateDir(chan_path.clone()));
                                tx.send(
                                    ControlCommand::Message(
                                        chan_path.clone().join("out"),
                                        format!("{}: {}\n",
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
            }
        });
    }
}

impl FilesystemMT for IrcFs {
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
        let mut fs = self.fs.write().unwrap();

        fs.mk_rw_file("/in").unwrap();
        fs.mk_ro_file("/out").unwrap();


        for srv_conf in convert_config(&self.config).into_iter() {
            self.handle_server(srv_conf);
        }

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
enum ControlCommand {
    CreateDir(PathBuf),
    Message(PathBuf, Vec<u8>),
}
