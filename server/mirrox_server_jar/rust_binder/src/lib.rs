mod binder_utils;
mod media_projection;
mod binder_transaction;

use jni::objects::{JClass, JValue};
use jni::sys::jobject;
use jni::JNIEnv;
use log::{error, info};

#[no_mangle]
pub extern "system" fn Java_com_mirrox_server_StartMirrox_getMediaProjectionTokenNative(
    mut env: JNIEnv,
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

            // Step 1: Find android.os.Binder class
            let binder_class = match env.find_class("android/os/Binder") {
                Ok(cls) => cls,
                Err(e) => {
                    error!("❌ Failed to find Binder class: {:?}", e);
                    return std::ptr::null_mut();
                }
            };

            // Step 2: Create new Binder instance
            let binder_obj = match env.new_object(binder_class, "()V", &[]) {
                Ok(obj) => obj,
                Err(e) => {
                    error!("❌ Failed to create Binder instance: {:?}", e);
                    return std::ptr::null_mut();
                }
            };

            // Step 3: Call nativeInit(long) on the Binder instance
            if let Err(e) = env.call_method(
                &binder_obj,
                "nativeInit",
                "(J)V",
                &[JValue::Long(token_ptr as i64)],
            ) {
                error!("❌ Failed to call nativeInit: {:?}", e);
                return std::ptr::null_mut();
            }

            // Step 4: Return raw object
            return binder_obj.into_raw();
        }
        None => {
            error!("❌ Failed to acquire MediaProjection token");
            std::ptr::null_mut()
        }
    }
}
