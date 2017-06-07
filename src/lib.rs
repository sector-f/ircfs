extern crate serde;
#[macro_use] extern crate serde_derive;

extern crate fuse;
extern crate fuse_mt;
extern crate time;
extern crate libc;
extern crate irc;
extern crate num_cpus;

pub mod config;
// pub mod ircfs;
// pub mod filesystem;

// pub mod tree_fs;
pub mod mem_safe_fs;
pub mod mem_safe_ircfs;
pub mod permissions;
