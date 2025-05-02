use std::ffi::CString;
use libc::{SYS_openat, O_RDWR, O_CLOEXEC};
use std::io::{Error, Result};
use std::os::unix::io::RawFd;
use nix::libc;

pub fn open_binder_device() -> Result<RawFd> {
    let path = CString::new("/dev/binder").unwrap();
    let fd = unsafe { libc::syscall(SYS_openat, path.as_ptr(), O_RDWR | O_CLOEXEC) as RawFd };

    if fd < 0 {
        Err(Error::last_os_error())
    } else {
        Ok(fd)
    }
}
