use std::os::unix::io::RawFd;
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;

pub fn open_binder_device() -> nix::Result<RawFd> {
    open("/dev/binder", OFlag::O_RDWR | OFlag::O_CLOEXEC, Mode::empty())
}
