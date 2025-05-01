mod binder_transaction;
mod binder_utils;

use std::io::Error;

fn main() -> Result<(), Error> {
    use binder_utils::open_binder_device;
    use binder_transaction::{send_create_projection, receive_binder_reply};

    let fd = open_binder_device()?;
    println!("[*] Opened /dev/binder (fd = {})", fd);

    let manager_handle = 3;
    println!("[*] Using MediaProjectionManager handle: {}", manager_handle);

    send_create_projection(fd, manager_handle)?;
    println!("[*] Sent createProjection transaction");

    let projection_token = receive_binder_reply(fd)?;
    println!("[*] Received MediaProjection token (IBinder handle): {}", projection_token);

    println!("[+] Done!");
    Ok(())
}
