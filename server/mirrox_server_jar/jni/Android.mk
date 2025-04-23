LOCAL_PATH := $(call my-dir)

include $(CLEAR_VARS)

LOCAL_MODULE    := mirroxjni
LOCAL_SRC_FILES := media_projection.c binder_utils.c
LOCAL_C_INCLUDES := $(LOCAL_PATH)/include

LOCAL_LDLIBS := -llog

include $(BUILD_SHARED_LIBRARY)
