extern crate serde;
#[macro_use] extern crate serde_derive;

extern crate fuse;
extern crate time;
extern crate libc;
extern crate irc;

pub mod config;
pub mod ircfs;
pub mod filesystem;
