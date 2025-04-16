// media_projection.h
#ifndef MIRROX_MEDIA_PROJECTION_H
#define MIRROX_MEDIA_PROJECTION_H

#include <jni.h>

JNIEXPORT jobject JNICALL
Java_com_mirrox_server_StartMirrox_getMediaProjectionTokenNative(JNIEnv *env, jclass clazz);

#endif
