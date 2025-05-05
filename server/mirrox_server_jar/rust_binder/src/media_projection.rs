use crate::binder_utils::open_binder_device;
use crate::binder_transaction::{send_create_projection, receive_binder_reply};
use log::{info, error};

/// Attempts to create a MediaProjection token via Binder IPC.
///
/// Returns the Binder object pointer (`u32`) on success.
pub fn create_media_projection_token() -> Option<u32> {
    let fd = open_binder_device().ok()?;
    let handle = 3; // Well-known handle for IMediaProjectionManager

    if send_create_projection(fd, handle).is_err() {
        error!("❌ Failed to send createProjection transaction");
        return None;
    }

    info!("✔ Sent createProjection transaction");

    match receive_binder_reply(fd) {
        Ok(ptr) => {
            info!("🎉 Received Binder object pointer: 0x{:X}", ptr);
            Some(ptr)
        }
        Err(e) => {
            error!("⛔ Failed to receive binder reply: {}", e);
            None
        }
    }
}
