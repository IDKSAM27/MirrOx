use std::os::unix::io::RawFd;
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::jobject;
use log::error;

#[no_mangle]
pub extern "system" fn Java_com_mirrox_server_StartMirrox_getMediaProjectionTokenNative(
    mut env: JNIEnv,
    _class: JClass,
) -> jobject {
    // Initialize logging first
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("MirrOxRust"),
    );

    // Dummy Binder object (safe stub)
    let binder_class = match env.find_class("android/os/Binder") {
        Ok(c) => c,
        Err(e) => {
            error!("❌ Failed to find android.os.Binder: {:?}", e);
            return std::ptr::null_mut();
        }
    };

    let binder_obj = match env.new_object(binder_class, "()V", &[]) {
        Ok(o) => o,
        Err(e) => {
            error!("❌ Failed to create Binder object: {:?}", e);
            return std::ptr::null_mut();
        }
    };

    // Log binder init steps
    match open_binder() {
        Ok(binder_fd) => {
            if send_create_projection_transaction(binder_fd).is_ok() {
                error!("✅ Success sending createProjection transaction!");
            } else {
                error!("❌ Failed to send createProjection");
            }
        }
        Err(e) => {
            error!("❌ Failed to open binder device: {:?}", e);
        }
    }

    // Return dummy binder (used as placeholder for now)
    binder_obj.into_raw()
}

fn open_binder() -> nix::Result<RawFd> {
    open("/dev/binder", OFlag::O_RDWR | OFlag::O_CLOEXEC, Mode::empty())
}

fn send_create_projection_transaction(_binder_fd: RawFd) -> nix::Result<()> {
    // This will be implemented later with full binder_transaction_data
    Ok(())
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn android_main() {
    std::panic::set_hook(Box::new(|info| {
        log::error!("🔥 Rust panic: {:?}", info);
    }));
}
