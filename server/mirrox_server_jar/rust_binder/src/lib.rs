mod binder_utils;
mod media_projection;
mod binder_transaction;

use jni::objects::JClass;
use jni::sys::jobject;
use jni::JNIEnv;
use log::error;

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

    log::info!("📡 Native getMediaProjectionTokenNative called");

    // Attempt to create MediaProjection token via raw Binder IPC
    match media_projection::create_media_projection_token() {
        Some(_) => log::info!("✅ Binder transaction succeeded"),
        None => error!("❌ Binder transaction failed"),
    }

    // For now, return a dummy Binder object
    let binder_class = env.find_class("android/os/Binder").unwrap();
    let binder_obj = env.new_object(binder_class, "()V", &[]).unwrap();
    binder_obj.into_raw()
}
