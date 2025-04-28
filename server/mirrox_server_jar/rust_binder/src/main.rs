mod binder; // <-- your binder.rs

use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::io::Error;

fn main() -> Result<(), Error> {
    // Step 1. Open binder device
    let binder_dev = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/binder")
        .expect("Failed to open /dev/binder");

    let fd = binder_dev.as_raw_fd();

    println!("[*] Opened /dev/binder (fd = {})", fd);

    // Step 2. Get the MediaProjectionManager service handle
    let media_projection_manager_handle = binder::receive_media_projection(fd)?;
    println!("[*] Got MediaProjectionManager handle: {}", media_projection_manager_handle);

    // Step 3. Send createProjection request
    binder::send_create_projection(fd, media_projection_manager_handle)?;
    println!("[*] Sent createProjection request");

    // Step 4. Receive the MediaProjection token
    let media_projection_token = binder::receive_media_projection(fd)?;
    println!("[*] Received MediaProjection token (IBinder handle): {}", media_projection_token);

    println!("[+] Done!");

    Ok(())
}
