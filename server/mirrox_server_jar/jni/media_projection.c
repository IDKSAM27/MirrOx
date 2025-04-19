#include <jni.h>
#include <stdio.h>
#include <binder/IServiceManager.h>
#include <binder/Parcel.h>
#include <utils/String16.h>
#include <utils/StrongPointer.h>
#include <android/log.h>

#define TAG "MirrOxJNI"
#define ALOGE(...) __android_log_print(ANDROID_LOG_ERROR, TAG, __VA_ARGS__)

using namespace android;

extern "C"
JNIEXPORT jobject JNICALL
Java_com_mirrox_server_StartMirrox_getMediaProjectionTokenNative(JNIEnv *env, jclass clazz) {
    sp<IBinder> service = defaultServiceManager()->getService(String16("media_projection"));
    if (service == nullptr) {
        ALOGE("❌ Failed to get media_projection service");
        return nullptr;
    }

    Parcel data, reply;
    data.writeInterfaceToken(String16("android.media.projection.IMediaProjectionManager"));

    int32_t uid = 2000; // shell UID (must be run via app_process as shell)
    String16 packageName("com.mirrox.server"); // Your server package name
    int32_t type = 1; // TYPE_SCREEN_CAPTURE
    bool permanentGrant = true;

    data.writeInt32(uid);
    data.writeString16(packageName);
    data.writeInt32(type);
    data.writeInt32(permanentGrant ? 1 : 0);

    status_t status = service->transact(5 /* TRANSACTION_createProjection */, data, &reply);
    if (status != NO_ERROR) {
        ALOGE("❌ transact failed: %d", status);
        return nullptr;
    }

    reply.readExceptionCode(); // ensure no remote exception
    sp<IBinder> projectionToken = reply.readStrongBinder();
    return env->NewLocalRef(reinterpret_cast<jobject>(projectionToken.get()));
}
