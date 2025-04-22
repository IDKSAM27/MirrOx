#include "media_projection.h"
#include "binder_utils.h"

#include <stdio.h>
#include <jni.h>
#include <android/log.h>

JNIEXPORT jobject JNICALL
Java_com_mirrox_server_StartMirrox_getMediaProjectionTokenNative(JNIEnv *env, jclass clazz) {
    // You can put native MediaProjection IPC or logging logic here
    printf("✅ Native startMediaProjection() called!\n");
    __android_log_print(ANDROID_LOG_INFO, "Mirrox", "✅ Native getMediaProjectionTokenNative() called!");
    
    // TODO: Replace this with real binder IPC logic
    return NULL; // Just to confirm flow works
}
