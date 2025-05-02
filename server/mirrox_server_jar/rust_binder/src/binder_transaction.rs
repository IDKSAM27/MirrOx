use std::mem;
use std::ptr;
use std::os::unix::io::RawFd;
use std::ffi::CString;
use nix::unistd::geteuid;
use nix::libc::{c_void, ioctl};
use std::error::Error;

// Required ioctl number for BINDER_WRITE_READ
const BINDER_WRITE_READ: u64 = 0xC0306201;
const BC_TRANSACTION: u32 = 0x5;
const TF_ONE_WAY: u32 = 0x01;

// Structs
#[repr(C)]
#[derive(Debug)]
struct BinderTransactionData {
    target: u64,          // handle or ptr
    cookie: u64,          // always 0
    code: u32,            // createProjection = 1
    flags: u32,           // e.g. TF_ONE_WAY
    data_buffer: u64,     // pointer to parcel buffer
    data_size: u64,
    offsets_buffer: u64,  // null (no binder refs)
    offsets_size: u64,
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

pub fn send_create_projection(fd: RawFd, manager_handle: u32) -> std::io::Result<()> {
    let iface = CString::new("android.media.projection.IMediaProjectionManager").unwrap();
    let pkg = CString::new("com.mirrox.server").unwrap();

    let mut parcel = Vec::new();

    // === AIDL Parcel format ===
    parcel.extend_from_slice(&(0x01000000u32).to_le_bytes()); // strict mode policy
    parcel.extend_from_slice(&(iface.to_bytes_with_nul().len() as u32).to_le_bytes());
    parcel.extend_from_slice(iface.to_bytes_with_nul());
    parcel.resize(4 + 4 + 128, 0); // pad interface_token
    parcel.extend_from_slice(&(geteuid().as_raw() as u32).to_le_bytes()); // UID
    parcel.extend_from_slice(&(pkg.to_bytes_with_nul().len() as u32).to_le_bytes());
    parcel.extend_from_slice(pkg.to_bytes_with_nul());
    parcel.resize(parcel.len() + (128 - pkg.to_bytes_with_nul().len()), 0); // pad package
    parcel.extend_from_slice(&0u32.to_le_bytes()); // flags (0)
    parcel.extend_from_slice(&0u32.to_le_bytes()); // unused

    // Allocate buffer in memory
    let parcel_ptr = parcel.as_ptr() as u64;
    let parcel_len = parcel.len() as u64;

    let txn = BinderTransactionData {
        target: manager_handle as u64,
        cookie: 0,
        code: 1, // createProjection
        flags: TF_ONE_WAY,
        data_buffer: parcel_ptr,
        data_size: parcel_len,
        offsets_buffer: 0,
        offsets_size: 0,
    };

    // Create write buffer (BC_TRANSACTION followed by BinderTransactionData)
    let mut write_buf = Vec::new();
    write_buf.extend_from_slice(&BC_TRANSACTION.to_ne_bytes());
    write_buf.extend_from_slice(unsafe {
        std::slice::from_raw_parts(&txn as *const _ as *const u8, mem::size_of::<BinderTransactionData>())
    });

    // Create empty read buffer
    let mut read_buf = vec![0u8; 4096];

    // Set up binder_write_read
    let mut bwr = BinderWriteRead {
        write_size: write_buf.len() as u64,
        write_consumed: 0,
        write_buffer: write_buf.as_ptr() as u64,
        read_size: read_buf.len() as u64,
        read_consumed: 0,
        read_buffer: read_buf.as_mut_ptr() as u64,
    };

    // Perform ioctl
    let ret = unsafe {
        ioctl(fd, BINDER_WRITE_READ, &mut bwr as *mut _ as *mut c_void)
    };

    if ret < 0 {
        return Err(std::io::Error::last_os_error());
    }

    println!("✅ Binder transaction succeeded. Bytes read: {}", bwr.read_consumed);
    Ok(())
}

pub fn receive_binder_reply(fd: RawFd) -> Result<()> {
    let mut buffer = [0u8; 1024];
    let bytes = read(fd, &mut buffer).map_err(|e| Error::new(ErrorKind::Other, e))?;

    println!("[temp_receive_binder_reply] Read {} bytes", bytes);
    for (i, chunk) in buffer[..bytes].chunks(4).enumerate() {
        let val = u32::from_ne_bytes(chunk.try_into().unwrap_or([0, 0, 0, 0]));
        println!("  Word {}: 0x{:08x}", i, val);
    }

    Ok(())
}
