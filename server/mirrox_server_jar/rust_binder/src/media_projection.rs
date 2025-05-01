use crate::binder_utils::open_binder_device;
use crate::binder_transaction::{send_create_projection, receive_binder_reply};
use log::info;

pub fn create_media_projection_token() -> Option<()> {
    let fd = open_binder_device().ok()?;
    let handle = 3; // default IMediaProjectionManager handle in system_service_manager

    if send_create_projection(fd, handle).is_ok() {
        info!("✔ Sent createProjection transaction");

        match receive_binder_reply(fd) {
            Ok(h) => {
                info!("🎉 Received handle: {}", h);
                Some(())
            }
            Err(e) => {
                log::error!("⛔ Failed to receive binder reply: {}", e);
                None
            }
        }
    } else {
        log::error!("❌ Failed to send transaction");
        None
    }
}
