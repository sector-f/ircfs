# ircfs
IRC filesystem based on [Suckless' ii](http://tools.suckless.org/ii/)

## About

`ircfs` uses [FUSE](https://github.com/libfuse/libfuse) to map IRC servers to directories on your computer:

```
freenode
├── #bash
│   ├── in
│   └── out
├── ##linux
│   ├── in
│   └── out
├── in
└── out
```

Each channel appears as a directory under the mountpoint. Text written to the channel's `in` file (e.g. `echo "Hello!" > '##linux/in'`) is sent as a message to that channel. Messages in the channel can be read via the `out` file.

## Usage

`ircfs -s SERVER -n NICKNAME MOUNTPOINT`

Alternatively, a TOML configuration file can be provided and specified with the `-c` flag. An example is provided at `examples/ircfs.toml`. If flags are used in addition to a configuration file, then the flags take precedence.

## Current Functionality

`ircfs` is very much a work-in-progress, and its functionality is therefore very limited at the moment.

The following has been implemented:

* Messages can be sent to and received from channels
* Channels can be joined by writing `/j #channelname` to any `in` file
* Private messaging (AKA queries) can be done, but the user must message you first

## Planned Functionality

* Messages from the server being written to the server's `out` file
* Support for connecting via SSL
* Full support for private messaging
* Standard IRC commands: `/me`, `/kick`, `/part`, `quit`, etc.

Additionally, `ircfs` may eventually be modified so that one `ircfs` instance handles connections to multiple IRC servers.

## Comparison to `ii`

### Pros

* `ii` uses a regular file for output. This means that a disk write occurs for every single message from the IRC server. That is in no way optimal. `ircfs` instead stores all messages in memory.
* `ii` uses a FIFO file for input. This means that messages sent to the `in` file are
essentially lost; no record of them is kept (aside from your shell's history, perhaps).
`ircfs` saves the data written to the `in` files so that you may read from them if desired.

### Cons

* Since `ircfs` stores message in memory rather than on an actual file, creating permanent logs would need to be done via some external means (such as a cron job copying the file to some other location)
