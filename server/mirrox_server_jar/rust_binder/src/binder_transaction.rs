use nix::unistd::{write, read, geteuid};
use std::os::unix::io::RawFd;
use std::io::{Error, ErrorKind, Result};
use std::ffi::CString;
use std::mem;

// Binder driver commands
const BR_TRANSACTION: u32 = 0x0C;
const BC_TRANSACTION: u32 = 0x5;

// Transaction flags
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
    offsets_buffer: u64,
    offsets_size: u64,
}

#[repr(C)]
#[derive(Debug)]
struct TransactionData {
    strict_mode_policy: u32,
    interface_token_len: u32,
    interface_token: [u8; 128],
    uid: u32,
    package_name_len: u32,
    package_name: [u8; 128],
    param3: u32,
    param4: u32,
}

#[repr(C)]
#[derive(Debug)]
struct BinderWriteBuf {
    cmd: u32,
    txn: BinderTransactionData,
}

pub fn send_create_projection(fd: RawFd, manager_handle: u32) -> Result<()> {
    let iface = CString::new("android.media.projection.IMediaProjectionManager").unwrap();
    let package = CString::new("com.mirrox.server").unwrap();

    let mut data = TransactionData {
        strict_mode_policy: 0x01000000,
        interface_token_len: iface.to_bytes_with_nul().len() as u32,
        interface_token: [0; 128],
        uid: geteuid().as_raw() as u32,
        package_name_len: package.to_bytes_with_nul().len() as u32,
        package_name: [0; 128],
        param3: 0,
        param4: 0,
    };

    data.interface_token[..iface.to_bytes_with_nul().len()]
        .copy_from_slice(iface.to_bytes_with_nul());
    data.package_name[..package.to_bytes_with_nul().len()]
        .copy_from_slice(package.to_bytes_with_nul());

    let txn_data = BinderTransactionData {
        target: manager_handle as u64,
        cookie: 0,
        code: 1, // createProjection
        flags: TF_ONE_WAY,
        data_buffer: 0,
        data_size: mem::size_of::<TransactionData>() as u64,
        offsets_buffer: 0,
        offsets_size: 0,
    };

    let write_buf = BinderWriteBuf {
        cmd: BC_TRANSACTION,
        txn: txn_data,
    };

    let mut final_buf = Vec::new();
    final_buf.extend_from_slice(unsafe {
        std::slice::from_raw_parts(
            &write_buf as *const _ as *const u8,
            mem::size_of::<BinderWriteBuf>(),
        )
    });

    final_buf.extend_from_slice(unsafe {
        std::slice::from_raw_parts(
            &data as *const _ as *const u8,
            mem::size_of::<TransactionData>(),
        )
    });

    write(fd, &final_buf).map_err(|e| Error::new(ErrorKind::Other, e))?;
    Ok(())
}

pub fn receive_binder_reply(fd: RawFd) -> Result<u32> {
    let mut buffer = [0u8; 1024];
    let bytes = read(fd, &mut buffer).map_err(|e| Error::new(ErrorKind::Other, e))?;

    if bytes < 4 {
        return Err(Error::new(ErrorKind::UnexpectedEof, "No data"));
    }

    let mut i = 0;
    while i + 4 <= bytes {
        let cmd = u32::from_ne_bytes(buffer[i..i+4].try_into().unwrap());
        i += 4;

        if cmd == BR_TRANSACTION {
            if i + mem::size_of::<BinderTransactionData>() > bytes {
                return Err(Error::new(ErrorKind::UnexpectedEof, "No transaction struct"));
            }

            let txn: &BinderTransactionData = unsafe {
                &*(buffer[i..].as_ptr() as *const BinderTransactionData)
            };

            let data_pos = i + mem::size_of::<BinderTransactionData>();
            if data_pos + 4 > bytes {
                return Err(Error::new(ErrorKind::UnexpectedEof, "No handle found"));
            }

            let handle = u32::from_ne_bytes(buffer[data_pos..data_pos + 4].try_into().unwrap());
            return Ok(handle);
        }

        i += mem::size_of::<BinderTransactionData>();
    }

    Err(Error::new(ErrorKind::NotFound, "No BR_TRANSACTION found"))
}
