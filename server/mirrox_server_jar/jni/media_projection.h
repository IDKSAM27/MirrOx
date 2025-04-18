// media_projection.h
// #ifndef MIRROX_MEDIA_PROJECTION_H
// #define MIRROX_MEDIA_PROJECTION_H

// #include <jni.h>

// JNIEXPORT jobject JNICALL
// Java_com_mirrox_server_StartMirrox_getMediaProjectionTokenNative(JNIEnv *env, jclass clazz);

// #endif


/* DO NOT include in build if you're already compiling the C file directly */

#ifndef _Included_com_mirrox_server_StartMirrox
#define _Included_com_mirrox_server_StartMirrox
#ifdef __cplusplus
extern "C" {
#endif
JNIEXPORT jint JNICALL Java_com_mirrox_server_StartMirrox_startMediaProjection(JNIEnv *, jclass);
#ifdef __cplusplus
}
#endif
#endif
