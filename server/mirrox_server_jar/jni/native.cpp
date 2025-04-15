#include <jni.h>
#include <binder/IServiceManager.h>
#include <binder/Parcel.h>
#include <utils/StrongPointer.h>
#include <android/log.h>

#define LOG_TAG "MirrOxNative"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

using namespace android;

extern "C"
JNIEXPORT jobject JNICALL
Java_com_mirrox_server_StartMirrox_nativeCreateProjection(JNIEnv* env, jclass clazz) {
    sp<IServiceManager> sm = defaultServiceManager();
    sp<IBinder> binder = sm->getService(String16("media_projection"));

    if (binder == nullptr) {
        LOGE("❌ Failed to get media_projection service");
        return nullptr;
    }

    Parcel data, reply;
    data.writeInterfaceToken(String16("android.media.projection.IMediaProjectionManager"));
    data.writeInt32(0); // uid = shell (0)
    data.writeString16(String16("com.mirrox.server")); // package name

    status_t result = binder->transact(1 /* createProjection */, data, &reply);
    if (result != NO_ERROR) {
        LOGE("❌ transact() failed: %d", result);
        return nullptr;
    }

    sp<IBinder> projection = reply.readStrongBinder();
    if (projection == nullptr) {
        LOGE("❌ Failed to get MediaProjection");
        return nullptr;
    }

    // Return as android.os.IBinder
    jclass binderClass = env->FindClass("android/os/BinderProxy");
    jmethodID constructor = env->GetMethodID(binderClass, "<init>", "()V");
    return env->NewLocalRef((jobject)projection.get());
}
