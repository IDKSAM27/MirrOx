use std::{
    io::{Error, ErrorKind, Result},
    mem::{size_of, ManuallyDrop},
    os::unix::io::RawFd,
    ptr,
};

use nix::libc::{c_void, geteuid, getpid};
use std::ffi::CString;
use libc::ioctl;

const BC_TRANSACTION: u32 = 0x40046301;
const BR_REPLY: u32 = 0x3;
const BINDER_WRITE_READ: i32 = 0xC0306201u32 as i32;
const TRANSACTION_CREATE_PROJECTION: u32 = 1;

#[repr(C)]
struct BinderWriteRead {
    write_size: u64,
    write_consumed: u64,
    write_buffer: u64,
    read_size: u64,
    read_consumed: u64,
    read_buffer: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct binder_transaction_data_ptr {
    buffer: u64,
    offsets: u64,
}

#[repr(C)]
union binder_transaction_data_data {
    ptr: ManuallyDrop<binder_transaction_data_ptr>,
    buf: [u8; 8],
}

#[repr(C)]
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

pub fn send_create_projection(fd: RawFd, manager_handle: u32) -> Result<()> {
    println!("📡 [Rust] Starting send_create_projection()");

    let iface = CString::new("android.media.projection.IMediaProjectionManager").unwrap();
    let pkg = CString::new("com.mirrox.server").unwrap();

    let mut parcel = Vec::new();

    // === Parcel construction ===
    parcel.extend_from_slice(&0x01000000u32.to_le_bytes()); // Strict mode header
    parcel.extend_from_slice(&(iface.to_bytes_with_nul().len() as u32).to_le_bytes());
    parcel.extend_from_slice(iface.to_bytes_with_nul());

    // Pad to 128 bytes total
    let current_len = parcel.len();
    if current_len < 128 {
        parcel.resize(128, 0);
    }

    let uid = unsafe { geteuid() } as u32;
    let pid = unsafe { getpid() } as u32;

    println!("🆔 UID: {}, PID: {}", uid, pid);

    parcel.extend_from_slice(&uid.to_le_bytes());

    parcel.extend_from_slice(&(pkg.to_bytes_with_nul().len() as u32).to_le_bytes());
    parcel.extend_from_slice(pkg.to_bytes_with_nul());

    // Pad package name to next 4-byte boundary if needed
    while parcel.len() % 4 != 0 {
        parcel.push(0);
    }

    parcel.extend_from_slice(&0u32.to_le_bytes()); // flags
    parcel.extend_from_slice(&0u32.to_le_bytes()); // unused int

    println!("📦 Final Parcel Size: {} bytes", parcel.len());

    // === Transaction data ===
    let txn_data = binder_transaction_data {
        target: manager_handle as u64,
        cookie: 0,
        code: TRANSACTION_CREATE_PROJECTION,
        flags: 0, // no TF_ONE_WAY
        sender_pid: pid,
        sender_euid: uid,
        data_size: parcel.len() as u64,
        offsets_size: 0,
        data: binder_transaction_data_data {
            ptr: ManuallyDrop::new(binder_transaction_data_ptr {
                buffer: parcel.as_ptr() as u64,
                offsets: 0,
            }),
        },
    };

    // === Write buffer ===
    let mut write_buf = Vec::new();
    write_buf.extend_from_slice(&BC_TRANSACTION.to_ne_bytes());
    write_buf.extend_from_slice(unsafe {
        std::slice::from_raw_parts(
            &txn_data as *const _ as *const u8,
            size_of::<binder_transaction_data>(),
        )
    });

    // === Read buffer ===
    let mut read_buf = vec![0u8; 4096];

    let mut bwr = BinderWriteRead {
        write_size: write_buf.len() as u64,
        write_consumed: 0,
        write_buffer: write_buf.as_ptr() as u64,
        read_size: read_buf.len() as u64,
        read_consumed: 0,
        read_buffer: read_buf.as_mut_ptr() as u64,
    };

    println!("🚀 Sending ioctl(BINDER_WRITE_READ)...");
    let ret = unsafe { ioctl(fd, BINDER_WRITE_READ as _, &mut bwr as *mut _ as *mut c_void) };

    if ret < 0 {
        return Err(Error::last_os_error());
    }

    println!("✅ Transaction sent. Bytes read: {}", bwr.read_consumed);

    if bwr.read_consumed > 0 {
        println!(
            "📥 First reply bytes: {:02X?}",
            &read_buf[..std::cmp::min(bwr.read_consumed as usize, 32)]
        );
    }

    Ok(())
}

pub fn receive_binder_reply(fd: RawFd) -> Result<u32> {
    println!("📡 [Rust] Waiting for binder reply...");

    let mut read_buf = vec![0u8; 4096];
    let mut bwr = BinderWriteRead {
        write_size: 0,
        write_consumed: 0,
        write_buffer: 0,
        read_size: read_buf.len() as u64,
        read_consumed: 0,
        read_buffer: read_buf.as_mut_ptr() as u64,
    };

    let ret = unsafe {
        ioctl(fd, BINDER_WRITE_READ, &mut bwr as *mut _ as *mut c_void)
    };

    if ret < 0 {
        return Err(Error::last_os_error());
    }

    let bytes_read = bwr.read_consumed as usize;
    println!("📩 Binder reply received ({} bytes)", bytes_read);

    let mut offset = 0;
    while offset + 4 <= bytes_read {
        let cmd = u32::from_ne_bytes(read_buf[offset..offset + 4].try_into().unwrap());
        offset += 4;

        match cmd {
            BR_REPLY => {
                println!("🎁 Received BR_REPLY");

                if offset + size_of::<binder_transaction_data>() > bytes_read {
                    return Err(Error::new(ErrorKind::UnexpectedEof, "Incomplete BR_REPLY data"));
                }

                let txn: binder_transaction_data = unsafe {
                    ptr::read_unaligned(read_buf[offset..].as_ptr() as *const _)
                };
                offset += size_of::<binder_transaction_data>();

                let ptr = unsafe { txn.data.ptr.buffer };
                println!("🔗 Returned binder object handle: 0x{:x}", ptr);
                return Ok(ptr as u32);
            }
            unknown => {
                println!("⚠️ Unknown binder command: 0x{:x}", unknown);
                break;
            }
        }
    }

    Err(Error::new(ErrorKind::Other, "No valid BR_REPLY received"))
}
