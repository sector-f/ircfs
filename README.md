# ircfs
IRC filesystem based on [Suckless' ii](http://tools.suckless.org/ii/)

## About

`ircfs` uses [FUSE](https://github.com/libfuse/libfuse) to map IRC servers to directories on your computer. Each channel appears as a directory under the mountpoint:

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

## Usage

`ircfs -s SERVER -n NICKNAME MOUNTPOINT`

Alternatively, a TOML configuration file can be provided and specified with the `-c` flag. An example `ircfs.toml` file is provided.
If flags are used in addition to a configuration file, then the flags take precedence.

Messages are sent by writing data to the `in` file for a channel or user: `echo "How do I install Gentoo?" > '##linux/in'`

Messages can be read via the corresponding `out` file.

Commands are performed by writing to the server's `in` file. The following commands have been implemented:

* `/join CHANNELS [KEYS]`: Joins the comma-separated list of channels, using the (optional) comma-separated list of keys.
* `/msg TARGET [MESSAGE]`: Sends a message to the target, whether it's a channel or user.
  If no message is specified, this creates a directory for the target without sending a message.
* `/part TARGETS`: Parts the comma-separated list of channels.

## Functionality

### Current Functionality

`ircfs` is very much a work-in-progress, and its functionality is therefore very limited at the moment.

The following has been implemented:

* Channels can be joined
* Messages can be sent to and received from channels/users

### Planned Functionality

* Support for connecting via SSL
* Standard IRC commands: `/me`, `/kick`, `/part`, `/quit`, etc.

Additionally, `ircfs` may eventually be modified so that one `ircfs` instance handles connections to multiple IRC servers.

## Comparison to `ii`

### Pros

* `ii` uses a regular file for output. This means that a disk write occurs for every single message from the IRC server. That is in no way optimal. `ircfs` instead stores all messages in memory.
* `ii` uses a FIFO file for input. This means that messages sent to the `in` file are
essentially lost; no record of them is kept (aside from your shell's history, perhaps).
`ircfs` saves the data written to the `in` files so that you may read from them if desired.

### Cons

* Since `ircfs` stores message in memory rather than on an actual file, creating permanent logs would need to be done via some external means (such as a cron job copying the file to some other location)
