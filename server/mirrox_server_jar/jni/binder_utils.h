#ifndef BINDER_UTILS_H
#define BINDER_UTILS_H

#include <jni.h>

int open_binder(void);
int send_create_projection_transaction(int binder_fd, uint32_t handle);
jobject receive_media_projection_reply(int binder_fd, JNIEnv *env);

#endif
