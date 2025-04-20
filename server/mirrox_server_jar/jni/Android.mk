LOCAL_PATH := $(call my-dir)

include $(CLEAR_VARS)

LOCAL_MODULE    := mirroxjni
LOCAL_SRC_FILES := media_projection.c

include $(BUILD_SHARED_LIBRARY)
