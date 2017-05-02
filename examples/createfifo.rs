extern crate nix;
use nix::sys::stat::Mode;

extern crate riiir;
use riiir::fifo;

fn main() {
    let perm = Mode::from_bits(0o644).unwrap();
    fifo::mkfifo("test.fifo", perm).unwrap();
}
