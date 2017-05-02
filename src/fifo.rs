use nix;
use nix::sys::stat::{mknod, Mode, S_IFIFO};
use nix::NixPath;

pub fn mkfifo<P: ?Sized + NixPath>(path: &P, perm: Mode) -> nix::Result<()> {
    mknod(path, S_IFIFO, perm, 0)
}
