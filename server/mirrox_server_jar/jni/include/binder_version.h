#ifndef MIRROX_BINDER_VERSION_H
#define MIRROX_BINDER_VERSION_H
#define IMEDIAPROJECTIONMANAGER_DESCRIPTOR "android.media.projection.IMediaProjectionManager"
#define IMEDIAPROJECTIONMANAGER_CREATE_PROJECTION_TRANSACTION 1  // method index for createProjection() 

#include <stdint.h>

struct binder_version {
    int32_t protocol_version;
};

#endif // MIRROX_BINDER_VERSION_H
