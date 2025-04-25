#ifndef MIRROX_BINDER_VERSION_H
#define MIRROX_BINDER_VERSION_H
#define IMEDIAPROJECTIONMANAGER_DESCRIPTOR "android.media.projection.IMediaProjectionManager"
#define IMEDIAPROJECTIONMANAGER_CREATE_PROJECTION_TRANSACTION 1  // method index for createProjection() 

#include <stdint.h>

struct binder_version {
    int32_t protocol_version;
};
int send_create_projection_transaction(int binder_fd, uint32_t handle);
jobject receive_media_projection_reply(int binder_fd, JNIEnv *env);


#endif // MIRROX_BINDER_VERSION_H
