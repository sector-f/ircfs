extern crate ircfs;
use ircfs::filesystem::*;

use std::path::Path;

#[allow(unused_mut)]
fn main() {
    let mut root = IrcDir::new(1, 0, 0);

    root.insert_node(
        Path::new("foo"),
        Node::D(IrcDir::new(2, 0, 0)),
    ).unwrap();

    root.insert_node(
        Path::new("bar"),
        Node::D(IrcDir::new(3, 0, 0)),
    ).unwrap();

    root.insert_node(
        Path::new("bar/barfile"),
        Node::F(IrcFile::new(4, 0, 0)),
    ).unwrap();

    root.insert_node(
        Path::new("bar/grandchild"),
        Node::D(IrcDir::new(5, 0, 0)),
    ).unwrap();

    root.insert_node(
        Path::new("bar/grandchild/great_grandchild"),
        Node::D(IrcDir::new(6, 0, 0)),
    ).unwrap();

    println!("{}", root.attr().ino);
    println!("{}", root.get(Path::new("foo/")).unwrap().attr().ino);
    println!("{}", root.get(Path::new("bar")).unwrap().attr().ino);
    println!("{}", root.get(Path::new("bar/barfile")).unwrap().attr().ino);
    println!("{}", root.get(Path::new("bar/grandchild")).unwrap().attr().ino);
    println!("{}", root.get(Path::new("bar/grandchild/great_grandchild")).unwrap().attr().ino);

    if let Some(_node) = root.get(Path::new("invalid")) {
        println!("1 is wrong if you can see this message");
    }

    if let Some(_node) = root.get(Path::new("bar/invalid")) {
        println!("2 is wrong if you can see this message");
    }

    if let Some(_node) = root.get(Path::new("bar/barfile/invalid")) {
        println!("3 is wrong if you can see this message");
    }

    if let Some(_node) = root.get(Path::new("")) {
        println!("4 is wrong if you can see this message");
    }

    if let Some(_node) = root.get(Path::new("foo/asdf/foo")) {
        println!("5 is wrong if you can see this message");
    }
}
