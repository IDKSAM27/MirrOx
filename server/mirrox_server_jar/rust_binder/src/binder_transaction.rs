use std::{
    ffi::CString,
    io::{Error, Result},
    mem,
    os::unix::io::RawFd,
    ptr,
};
use nix::libc::{geteuid, ioctl, c_void};
use std::io::ErrorKind;

// These must match kernel definitions
const BR_REPLY: u32 = 0x3;
const BC_TRANSACTION: u32 = 0x5;
const BINDER_WRITE_READ: u32 = 0xC0306201;
const TRANSACTION_createProjection: u32 = 1;

#[repr(C)]
#[derive(Debug)]
struct binder_transaction_data {
    target: u64,
    cookie: u64,
    code: u32,
    flags: u32,
    sender_pid: u32,
    sender_euid: u32,
    data_size: u64,
    offsets_size: u64,
    data: binder_transaction_data_data,
}

#[repr(C)]
#[derive(Debug)]
union binder_transaction_data_data {
    ptr: binder_transaction_data_ptr,
    buf: [u8; 8],
}

#[repr(C)]
#[derive(Debug)]
struct binder_transaction_data_ptr {
    buffer: u64,
    offsets: u64,
}

#[repr(C)]
#[derive(Debug)]
struct BinderWriteRead {
    write_size: u64,
    write_consumed: u64,
    write_buffer: u64,
    read_size: u64,
    read_consumed: u64,
    read_buffer: u64,
}

pub fn send_create_projection(fd: RawFd, manager_handle: u32) -> Result<()> {
    let iface = CString::new("android.media.projection.IMediaProjectionManager").unwrap();
    let pkg = CString::new("com.mirrox.server").unwrap();

    let mut parcel = Vec::new();

    // === Parcel setup ===
    parcel.extend_from_slice(&0x01000000u32.to_le_bytes()); // Strict mode policy
    parcel.extend_from_slice(&(iface.to_bytes_with_nul().len() as u32).to_le_bytes());
    parcel.extend_from_slice(iface.to_bytes_with_nul());
    parcel.resize(4 + 4 + 128, 0); // pad interface_token to 128

    parcel.extend_from_slice(&(unsafe { geteuid() } as u32).to_le_bytes());
    parcel.extend_from_slice(&(pkg.to_bytes_with_nul().len() as u32).to_le_bytes());
    parcel.extend_from_slice(pkg.to_bytes_with_nul());
    parcel.resize(parcel.len() + (128 - pkg.to_bytes_with_nul().len()), 0); // pad package
    parcel.extend_from_slice(&0u32.to_le_bytes()); // flags
    parcel.extend_from_slice(&0u32.to_le_bytes()); // unused

    println!("[*] Parcel buffer size: {}", parcel.len());

    // === Transaction ===
    let txn_data = binder_transaction_data {
        target: manager_handle as u64,
        cookie: 0,
        code: TRANSACTION_createProjection,
        flags: 0, // No TF_ONE_WAY
        sender_pid: 0,
        sender_euid: 0,
        data_size: parcel.len() as u64,
        offsets_size: 0,
        data: binder_transaction_data_data {
            ptr: binder_transaction_data_ptr {
                buffer: parcel.as_ptr() as u64,
                offsets: 0,
            }
        },
    };

    // === Write buffer ===
    let mut write_buf = Vec::new();
    write_buf.extend_from_slice(&BC_TRANSACTION.to_ne_bytes());
    write_buf.extend_from_slice(unsafe {
        std::slice::from_raw_parts(
            &txn_data as *const _ as *const u8,
            mem::size_of::<binder_transaction_data>(),
        )
    });

    // === Read buffer ===
    let mut read_buf = vec![0u8; 4096];

    let mut bwr = binder_write_read {
        write_size: write_buf.len() as u64,
        write_consumed: 0,
        write_buffer: write_buf.as_ptr() as u64,
        read_size: read_buf.len() as u64,
        read_consumed: 0,
        read_buffer: read_buf.as_mut_ptr() as u64,
    };

    // === IOCTL ===
    println!("[*] Sending binder transaction...");
    let ret = unsafe { ioctl(fd, BINDER_WRITE_READ as _, &mut bwr as *mut _ as *mut c_void) };

    if ret < 0 {
        return Err(Error::last_os_error());
    }

    println!("[+] Binder transaction sent! Read consumed: {}", bwr.read_consumed);

    // === Optional: Print some of the reply ===
    println!("[>] First few reply bytes: {:02X?}", &read_buf[..std::cmp::min(32, bwr.read_consumed as usize)]);

    Ok(())
}


pub fn receive_binder_reply(fd: RawFd) -> Result<u32> {
    let mut read_buf = vec![0u8; 4096];

    let mut bwr = BinderWriteRead {
        write_size: 0,
        write_consumed: 0,
        write_buffer: 0,
        read_size: read_buf.len() as u64,
        read_consumed: 0,
        read_buffer: read_buf.as_mut_ptr() as u64,
    };

    let ret = unsafe { ioctl(fd, BINDER_WRITE_READ, &mut bwr as *mut _ as *mut c_void) };

    if ret < 0 {
        return Err(Error::last_os_error());
    }

    let bytes_read = bwr.read_consumed as usize;
    println!("📥 Binder reply received ({} bytes)", bytes_read);

    let mut offset = 0;
    while offset + 4 <= bytes_read {
        let cmd = u32::from_ne_bytes(read_buf[offset..offset + 4].try_into().unwrap());
        offset += 4;

        match cmd {
            BR_REPLY => {
                println!("📦 Received BR_REPLY (0x{:x})", cmd);

                if offset + size_of::<binder_transaction_data>() > bytes_read {
                    return Err(Error::new(ErrorKind::UnexpectedEof, "Incomplete binder_transaction_data"));
                }

                let txn: binder_transaction_data = unsafe {
                    ptr::read_unaligned(read_buf[offset..].as_ptr() as *const _)
                };

                offset += size_of::<binder_transaction_data>();

                // Extract raw buffer pointer if returned
                let ptr = unsafe { txn.data.ptr.buffer };
                println!("📍 Returned binder object pointer: 0x{:x}", ptr);

                // You’ll need to parse the buffer to extract a real IBinder if it’s a binder reference
                return Ok(ptr as u32); // OR further parse the buffer!
            }
            unknown => {
                println!("⚠️ Unknown binder cmd: 0x{:x}", unknown);
                break; // or continue depending on expected flow
            }
        }
    }

    Err(Error::new(ErrorKind::Other, "No valid BR_REPLY received"))
}