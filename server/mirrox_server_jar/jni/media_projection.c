// media_projection.c
#include "media_projection.h"
#include <stdio.h>

#include <jni.h>
// #include <binder/IServiceManager.h>
// #include <binder/Parcel.h>
// #include <utils/String16.h>
// #include <utils/StrongPointer.h>

// using namespace android;

// Signature: Java_[package]_[ClassName]_methodname             , this is the only way to name the below function name
JNIEXPORT jobject JNICALL
Java_com_mirrox_server_StartMirrox_startMediaProjection(JNIEnv *env, jclass clazz) {
    // TODO: implement raw binder transaction like scrcpy
    printf("Native function called!\n");
    return 0;
}
