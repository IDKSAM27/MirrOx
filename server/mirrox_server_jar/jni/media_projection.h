// media_projection.h
// #ifndef MIRROX_MEDIA_PROJECTION_H
// #define MIRROX_MEDIA_PROJECTION_H

// #include <jni.h>

// JNIEXPORT jobject JNICALL
// Java_com_mirrox_server_StartMirrox_getMediaProjectionTokenNative(JNIEnv *env, jclass clazz);

// #endif


#ifndef MIRROX_MEDIA_PROJECTION_H
#define MIRROX_MEDIA_PROJECTION_H

#include <jni.h>  // ✅ required for JNIEXPORT and friends

#ifdef __cplusplus
extern "C" {
#endif

JNIEXPORT jint JNICALL
java_com_mirrox_server_StartMirrox_startMediaProjection(JNIEnv *env, jobject clazz);

#ifdef __cplusplus
}
#endif

#endif // MIRROX_MEDIA_PROJECTION_H
