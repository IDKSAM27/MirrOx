mod binder_utils;
mod media_projection;
mod binder_transaction;

use jni::objects::{JClass, JObject};
use jni::sys::jobject;
use jni::JNIEnv;
use log::{error, info};

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

    info!("📡 Native getMediaProjectionTokenNative called");

    match media_projection::create_media_projection_token() {
        Some(token_ptr) => {
            info!("✅ Received MediaProjection token ptr: 0x{:x}", token_ptr);

            // TODO: Convert this pointer to a usable Java Binder object.
            // For now, we still return a dummy android.os.Binder instance.
        }
        None => {
            error!("❌ Failed to acquire MediaProjection token");
        }
    }

    // Return a dummy `new Binder()` until token can be converted properly.
    let binder_class = match env.find_class("android/os/Binder") {
        Ok(class) => class,
        Err(e) => {
            error!("Failed to find android/os/Binder class: {:?}", e);
            return std::ptr::null_mut();
        }
    };

    let binder_obj = match env.new_object(binder_class, "()V", &[]) {
        Ok(obj) => obj,
        Err(e) => {
            error!("Failed to create Binder object: {:?}", e);
            return std::ptr::null_mut();
        }
    };

    binder_obj.into_raw()
}
