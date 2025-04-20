#ifndef MIRROX_MEDIA_PROJECTION_H
#define MIRROX_MEDIA_PROJECTION_H

#include <jni.h>  // ✅ required for JNIEXPORT and friends

#ifdef __cplusplus
extern "C" {
#endif

JNIEXPORT jint JNICALL
Java_com_mirrox_server_StartMirrox_startMediaProjection(JNIEnv *env, jobject thiz);

#ifdef __cplusplus
}
#endif

#endif // MIRROX_MEDIA_PROJECTION_H
