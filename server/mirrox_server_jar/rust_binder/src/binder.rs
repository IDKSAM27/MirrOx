use nix::unistd::{write, read};
use std::os::unix::io::RawFd;
use std::io::{Error, ErrorKind};
use std::mem;
use std::ffi::CString;
use nix::errno::Errno;

const BR_TRANSACTION: u32 = 0x0C; // 0x0C = 12 decimals
// Binder command constants
const BC_TRANSACTION: u32 = 0x5;
// Binder flags
const TF_ONE_WAY: u32 = 0x01;

#[repr(C)]
#[derive(Debug)]
struct BinderTransactionData {
    target: u64,
    cookie: u64,
    code: u32,
    flags: u32,
    data_buffer: u64,
    data_size: u64,
    offset_offset: u64,
    offsets_size: u64,
    data: [u8; 0], // Manually offset after the struct
}

#[repr(C, packed)]
struct TransactionData {
    strict_mode_policy: u32,
    interface_token_length: u32,
    interface_token: [u8; 128],
    uid: u32,
    package_name_length: u32,
    package_name: [u8; 128],
    param3: u32,
    param4: u32,
}

#[repr(C, packed)]
struct BinderWriteBuf {
    cmd: u32,
    txn: BinderTransactionData,
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

pub fn send_create_projection(fd: RawFd, manager_handle: u32) -> Result<(), std::io::Error> {
    let descriptor = CString::new("android.media.projection.IMediaProjectionManager").unwrap();
    let package_name = CString::new("com.mirrox.server").unwrap();

    let mut data = TransactionData {
        strict_mode_policy: 0x01000000,
        interface_token_length: descriptor.to_bytes_with_nul().len() as u32,
        interface_token: [0; 128],
        uid: nix::unistd::geteuid().as_raw() as u32,
        package_name_length: package_name.to_bytes_with_nul().len() as u32,
        package_name: [0; 128],
        param3: 0,
        param4: 0,
    };

    data.interface_token[..descriptor.to_bytes_with_nul().len()]
        .copy_from_slice(descriptor.to_bytes_with_nul());

    data.package_name[..package_name.to_bytes_with_nul().len()]
        .copy_from_slice(package_name.to_bytes_with_nul());

    let txn = BinderTransactionData {
        target: manager_handle as u64,
        cookie: 0,
        code: 1, // createProjection transaction code
        flags: TF_ONE_WAY,
        data_buffer: 0,
        data_size: mem::size_of::<TransactionData>() as u64,
        offsets_size: 0,
        data: [],
    };

    let write_buf = BinderWriteBuf {
        cmd: BC_TRANSACTION,
        txn,
    };

    // Build the final write buffer
    let mut raw_buf = Vec::new();
    raw_buf.extend_from_slice(unsafe {
        std::slice::from_raw_parts(
            &write_buf as *const _ as *const u8,
            mem::size_of::<BinderWriteBuf>(),
        )
    });

    raw_buf.extend_from_slice(unsafe {
        std::slice::from_raw_parts(
            &data as *const _ as *const u8,
            mem::size_of::<TransactionData>(),
        )
    });

    write(fd, &raw_buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(())
}