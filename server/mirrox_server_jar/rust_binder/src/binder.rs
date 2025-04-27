use nix::unistd::read;
use std::os::unix::io::RawFd;
use std::io::{Error, ErrorKind};

const BR_TRANSACTION: u32 = 0xC00000000;

#[repr(C)]
#[derive(Debug)]

struct BinderTransactionData {
    target: u64,
    cookie: u64,
    code: u32,
    flags: u32,
    data_buffer: u64,
    data_size: u64,
    offsets_size: u64,
    data: [u8; 0], // Manually offset after the struct
}

pub fn receive_media_projection(fd: RawFd) -> Result<u32, std::io::Error> {
    let mut buffer = [0u8; 1024];

    let bytes_read = read(fd, &mut buffer).map_err(|e| Error::new(ErrorKind::Other, e))?;
    if bytes_read < 4 {
        return Err(Error::new(ErrorKind::UnexpectedEof, "Not enough bytes for command"));
    }

    let mut pos = 0;

    while pos + 4 <= bytes_read {
        let cmd = u32::from_ne_bytes(buffer[pos..pos+4].try_into().unwrap());
        pos += 4;

        if cmd == BR_TRANSACTION {
            if pos + std::mem::size_of::<BinderTransactionData>() > bytes_read {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Incomplete BinderTransactionData"));
            }

            let txn_ptr = &buffer[pos] as *const u8 as *const BinderTransactionData;
            let txn = unsafe {&*txn_ptr};

            // Now after the txn struct, data follows (handle is first 4 bytes)
            let data_start = pos + std::mem::size_of::<BinderTransactionData>();

            if data_start + 4 > bytes_read {
                return Err(Error::new(ErrorKind::UnexpectedEof, "No binder handle found"));
            }

            let binder_handle = u32::from_ne_bytes(buffer[data_start..data_start+4].try_into().unwrap());

            return Ok(binder_handle);
            
        }

        pos += std::mem::size_of::<BinderTransactionData>(); // Skip unknown transaction
    }

    Err(Error::new(ErrorKind::NotFound, "No BR_TRANSACTION found!"))
}