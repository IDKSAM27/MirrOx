LOCAL_PATH := $(call my-dir)

include $(CLEAR_VARS)

LOCAL_MODULE    := mirroxjni
LOCAL_SRC_FILES := media_projection.c

LOCAL_LDLIBS := -llog -landroid

# Adjusted AOSP header includes
LOCAL_C_INCLUDES += \
    $(LOCAL_PATH)/../../platform_system_core/include \
    $(LOCAL_PATH)/../../platform_system_core/libutils/include \
    $(LOCAL_PATH)/../../platform_frameworks_base/core/java/android/os \
    $(LOCAL_PATH)/../../platform_frameworks_base/core/java/android/content \
    $(LOCAL_PATH)/../../platform_frameworks_base/media/java/android/media/projection

include $(BUILD_SHARED_LIBRARY)
