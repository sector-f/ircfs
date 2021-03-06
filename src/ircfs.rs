use libc::{ENOENT, EISDIR, ENOTSUP};
use time::{self, Timespec};

use irc::client::prelude::*;
use irc::error::Result as IrcResult;

use std::sync::{Arc, RwLock, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::path::{Path, PathBuf};
use std::ffi::OsString;
use std::thread;

use fuse_mt::*;
use filesystem::*;

pub struct IrcFs {
    fs: Arc<RwLock<Filesystem>>,
    server: IrcServer,
    tx_to_fs: Mutex<Sender<FsControl>>,
}

#[allow(unused_must_use)]
impl IrcFs {
    pub fn new(config: &Config, uid: u32, gid: u32) -> IrcResult<Self> {
        let mut config = config.clone();
        config.version = Some(format!("ircfs {}", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown version")));
        config.source = Some("https://github.com/sector-f/ircfs".to_owned());

        let srv = IrcServer::from_config(config.clone())?;

        let (tx, rx) = channel();

        let mut fs = Filesystem::new(uid, gid);

        fs.mk_rw_file("/send").unwrap();
        fs.mk_ro_file("/receive").unwrap();
        fs.mk_ro_file("/raw").unwrap();

        let filesystem = IrcFs {
            fs: Arc::new(RwLock::new(fs)),
            server: srv,
            tx_to_fs: Mutex::new(tx.clone()),
        };

        let fs = filesystem.fs.clone();
        thread::spawn(move || {
            for message in rx.iter() {
                let mut fs = fs.write().unwrap();
                match message {
                    FsControl::Message(ref path, ref data) => {
                        if let Some(&mut Node::F(ref mut file)) = fs.get_mut(path) {
                            file.insert_data(&data);
                        }
                    },
                    FsControl::CreateDir(ref path) => {
                        fs.mk_parents(&path);
                        fs.mk_ro_file(&path.join("receive"));
                        fs.mk_rw_file(&path.join("send"));
                    }
                }
            }
        });

        let tx_to_fs = tx.clone();
        let server = filesystem.server.clone();
        thread::spawn(move|| {
            let root = Path::new("/");
            server.identify();
            server.for_each_incoming(|msg| {
                let time = time::now();

                tx_to_fs.send(
                    FsControl::Message(
                        root.join("raw"),
                        format!("{} {}",
                            time.strftime("%T").unwrap(),
                            msg,
                        ).into_bytes(),
                    )
                );

                let msg_clone = msg.clone();
                match msg.command {
                    Command::PRIVMSG(target, message) => {
                        let username = msg_clone.source_nickname()
                            .unwrap_or(server.current_nickname()).to_owned();
                        let chan_path = if &target == server.current_nickname() {
                            root.join(&username)
                        } else {
                            root.join(target)
                        };
                        tx_to_fs.send(FsControl::CreateDir(chan_path.clone()));
                        tx_to_fs.send(
                            FsControl::Message(
                                chan_path.clone().join("receive"),
                                format!("{} {}: {}\n",
                                    time.strftime("%T").unwrap(),
                                    &username,
                                    message.trim(),
                                ).into_bytes(),
                            )
                        );
                    },
                    Command::JOIN(channel, _, _) => {
                        let username = msg_clone.source_nickname()
                            .unwrap_or(server.current_nickname()).to_owned();
                        let chan_path = root.join(&channel);
                        tx_to_fs.send(FsControl::CreateDir(chan_path.clone()));
                        tx_to_fs.send(
                            FsControl::Message(
                                chan_path.clone().join("receive"),
                                format!("{} {} has joined\n",
                                    time.strftime("%T").unwrap(),
                                    &username,
                                ).into_bytes(),
                            )
                        );
                    },
                    Command::PART(channel, reason) => {
                        let username = msg_clone.source_nickname()
                            .unwrap_or(server.current_nickname()).to_owned();
                        let chan_path = root.join(&channel);

                        let reason = if let Some(r) = reason {
                            format!(" ({})", r)
                        } else {
                            "".to_string()
                        };

                        tx_to_fs.send(FsControl::CreateDir(chan_path.clone()));
                        tx_to_fs.send(
                            FsControl::Message(
                                chan_path.clone().join("receive"),
                                format!("{} {} has left{}\n",
                                    time.strftime("%T").unwrap(),
                                    &username,
                                    &reason,
                                ).into_bytes(),
                            )
                        );
                    },
                    Command::PING(_, _) => {},
                    _ => {
                        tx_to_fs.send(
                            FsControl::Message(
                                root.join("receive"),
                                format!("{} {}",
                                    time.strftime("%T").unwrap(),
                                    msg,
                                ).into_bytes(),
                            )
                        );
                    },
                }
            });
        });

        if let Some(ref channels) = config.channels {
            let tx = filesystem.tx_to_fs.lock().unwrap();
            for channel in channels {
                let path = Path::new("/").join(channel);
                tx.send(FsControl::CreateDir(path));
            }
        }

        return Ok(filesystem);
    }
}

#[allow(unused_must_use)]
impl FilesystemMT for IrcFs {
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
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

                if offset >= data.len() as u64 {
                    Ok(Vec::new())
                } else {
                    let end = {
                        if (size as u64 + offset) as usize > data.len() {
                            data.len()
                        } else {
                            size as usize
                        }
                    };

                    if offset >= end as u64 {
                        Ok(Vec::new())
                    } else {
                        Ok(data[offset as usize..end].to_owned())
                    }
                }
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
        let time = time::now();

        match fs.get_mut(path) {
            Some(&mut Node::D(ref mut _dir)) => {
                Err(EISDIR)
            },
            Some(&mut Node::F(ref mut file)) => {
                let uid = file.attr.uid;
                let gid = file.attr.gid;
                let mode = file.attr.perm;
                let len = data.len();

                if can_write(uid, gid, mode, &req) {
                    if let Ok(mut string) = String::from_utf8(data) {
                        let trimmed_len = string.trim_right().len();
                        string.truncate(trimmed_len);
                        if string.is_empty() {
                            return Ok(len as u32);
                        }
                        string.push('\n');
                        file.insert_data(string.as_bytes());
                        let _ = string.pop();

                        if path == Path::new("/send") {
                            let sections = string.split(' ').collect::<Vec<_>>();
                            if let Some(command) = sections.iter().skip_while(|s| s.is_empty()).nth(0) {
                                let arguments = sections.iter().skip_while(|s| s.is_empty()).skip(1).skip_while(|s| s.is_empty()).map(|s| s.to_owned().trim()).collect::<Vec<_>>();
                                match *command {
                                    "/j" | "/join" | "j" | "join" => {
                                        if arguments.len() == 1 {
                                            let tx_to_fs = self.tx_to_fs.lock().unwrap();
                                            for chan in arguments[0].split(',') {
                                                let channel_path = Path::new("/").join(chan.clone());
                                                tx_to_fs.send(FsControl::CreateDir(channel_path.clone()));

                                                self.server.send_join(&chan);
                                            }
                                        } else if arguments.len() > 1 {
                                            let tx_to_fs = self.tx_to_fs.lock().unwrap();
                                            for (chan, key) in arguments[0].split(',').zip(arguments[1].split(',')) {
                                                let channel_path = Path::new("/").join(chan.clone());
                                                tx_to_fs.send(FsControl::CreateDir(channel_path.clone()));

                                                self.server.send_join_with_keys(&chan, &key);
                                            }
                                        }
                                    },
                                    "/part" | "part" => {
                                        if arguments.len() == 1 {
                                            for chan in arguments[0].split(',') {
                                                self.server.send(Message::from(Command::PART(String::from(chan), None)));
                                            }
                                        } else if arguments.len() > 1 {
                                            for (chan, reason) in arguments[0].split(',').zip(arguments[1].split(',')) {
                                                let r = if reason.is_empty() { None } else { Some(reason.to_owned()) };
                                                self.server.send(Message::from(Command::PART(String::from(chan), r)));
                                            }
                                        }
                                    },
                                    "/msg" | "msg" => {
                                        if arguments.len() == 1 {
                                            let tx_to_fs = self.tx_to_fs.lock().unwrap();
                                            let channel_path = Path::new("/").join(arguments[0].clone());
                                            tx_to_fs.send(FsControl::CreateDir(channel_path.clone()));
                                        } else if arguments.len() > 1 {
                                            let tx_to_fs = self.tx_to_fs.lock().unwrap();
                                            let channel_path = Path::new("/").join(arguments[0].clone());
                                            tx_to_fs.send(FsControl::CreateDir(channel_path.clone()));

                                            let message = arguments.iter().skip(1).map(|s| s.to_owned()).collect::<Vec<&str>>().join(" ");
                                            self.server.send_privmsg(arguments[0], &message);
                                            tx_to_fs.send(
                                                FsControl::Message(
                                                    channel_path.join("receive"),
                                                    format!("{} {}: {}\n",
                                                        time.strftime("%T").unwrap(),
                                                        self.server.current_nickname(),
                                                        message,
                                                    ).into_bytes(),
                                                )
                                            );
                                        }
                                    },
                                    _ => {},
                                }
                            }
                        } else {
                            let channel_dir = PathBuf::from(&path).parent().unwrap().to_owned();
                            let channel = channel_dir.file_name().unwrap();

                            self.server.send_privmsg(&channel.to_string_lossy(), &string);

                            let tx_to_fs = self.tx_to_fs.lock().unwrap();
                            tx_to_fs.send(
                                FsControl::Message(
                                    channel_dir.clone().join("receive"),
                                    format!("{} {}: {}\n",
                                        time.strftime("%T").unwrap(),
                                        self.server.current_nickname(),
                                        string.trim(),
                                    ).into_bytes(),
                                )
                            );
                        }
                    }

                    Ok(len as u32)
                } else {
                    // Should probably be changed to EACCES if/when permissions are implemented
                    // But, currently, this will just be the "receive" files, and ENOTSUP seems
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
