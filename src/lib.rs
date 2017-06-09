extern crate serde;
#[macro_use] extern crate serde_derive;

extern crate fuse;
extern crate fuse_mt;
extern crate time;
extern crate libc;
extern crate irc;
extern crate num_cpus;

pub mod ircfs;
pub mod filesystem;
pub mod permissions;
pub mod config;
