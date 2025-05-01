use std::fs::OpenOptions;
use std::os::unix::io::{AsRawFd, RawFd};
use std::io::{Error, ErrorKind};

pub fn open_binder_device() -> Result<RawFd, Error> {
    let binder = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/binder")
        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to open /dev/binder: {}", e)))?;

    Ok(binder.as_raw_fd())
}
