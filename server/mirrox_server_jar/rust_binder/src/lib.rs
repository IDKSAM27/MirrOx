use std::os::unix::io::RawFd;
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jobject;
use log::error;

#[warn(unused_variables)]
#[no_mangle]
pub extern "system" fn Java_com_mirrox_server_StartMirrox_getMediaProjectionTokenNative(
    env: JNIEnv,
    _class: JClass,
) -> jobject {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("MirrOxRust"),
    );

    match open_binder() {
        Ok(binder_fd) => {
            if send_create_projection_transaction(binder_fd).is_ok() {
                // TODO: receive reply
                error!("Success sending transaction!");
            } else {
                error!("Failed to send createProjection");
            }
        }
        Err(e) => {
            error!("Failed to open binder device: {:?}", e);
        }
    }

    std::ptr::null_mut()
}

fn open_binder() -> nix::Result<RawFd> {
    open("/dev/binder", OFlag::O_RDWR | OFlag::O_CLOEXEC, Mode::empty())
}

fn send_create_projection_transaction(_binder_fd: RawFd) -> nix::Result<()> {
    // TODO: Implement Binder protocol here
    Ok(())
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn android_main() {
    std::panic::set_hook(Box::new(|info| {
        log::error!("Rust panic: {:?}", info);
    }));
}
