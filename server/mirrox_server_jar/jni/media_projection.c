// media_projection.c
#include "media_projection.h"
#include <stdio.h>

#include <jni.h>

// Signature: Java_[package]_[ClassName]_methodname             , this is the only way to name the below function name
JNIEXPORT jobject JNICALL
Java_com_mirrox_server_StartMirrox_startMediaProjection(JNIEnv *env, jclass clazz) {
    // TODO: implement raw binder transaction like scrcpy
    printf("Native function called!\n");
    return 0;
}
