#include "media_projection.h"
#include <stdio.h>

JNIEXPORT jint JNICALL
Java_com_mirrox_server_StartMirrox_startMediaProjection(JNIEnv *env, jclass clazz) {
    // You can put native MediaProjection IPC or logging logic here
    printf("✅ Native startMediaProjection() called!\n");
    return 0;  // success
}
