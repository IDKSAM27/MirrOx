#include <jni.h>
#include <stdio.h>
#include "binder_utils.h"

JNIEXPORT jobject JNICALL
Java_com_mirrox_server_StartMirrox_getMediaProjectionTokenNative(JNIEnv *env, jclass clazz) {
    printf("✅ Native startMediaProjection() called!\n");

    int binder_fd = open_binder();
    if (binder_fd < 0) {
        printf("❌ Failed to open binder device\n");
        return NULL;
    }

    printf("🔧 create_media_projection() called (WIP)\n");

    // You will eventually perform the binder IPC transaction here to get the token
    //close(binder_fd);

    return NULL;
}
