#define _GNU_SOURCE

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <sys/ioctl.h>
#include <string.h>
#include <sys/syscall.h>
#include <jni.h>
#include <errno.h>
#include <sys/uio.h>

#include <android/log.h> // For Android logs

#include "include/binder.h"
#include "include/binderfs.h"
#include "include/ioctl.h"
#include "include/types.h"
#include "include/binder_version.h"
#include "binder_utils.h"

// 📢 Log error using printf (you can switch back to __android_log_print later if you want)
#define LOGE(fmt, ...) printf("[MirrOxJNI][ERROR] " fmt "\n", ##__VA_ARGS__)

// Binder constants
#define BC_TRANSACTION 0x5
#define BR_TRANSACTION 0xC0000000

// binder_transaction_data struct
struct binder_transaction_data {
    binder_uintptr_t target;
    binder_uintptr_t cookie;
    uint32_t code;
    uint32_t flags;
    union {
        struct {
            binder_uintptr_t ptr;
            binder_size_t length;
        } ptr;
        struct {
            uint64_t handle;
            uint64_t cookie;
        } u64;
    } data;
    binder_size_t offsets_size;
    binder_size_t data_size;
    binder_uintptr_t data_buffer;
};

// Open binder device
int open_binder() {
    int fd = syscall(SYS_openat, AT_FDCWD, BINDER_DEVICE, O_RDWR | O_CLOEXEC);
    if (fd < 0) {
        LOGE("open binder failed: %s", strerror(errno));
        return -1;
    }

    uint32_t vers = BINDER_CURRENT_PROTOCOL_VERSION;
    if (ioctl(fd, BINDER_VERSION, &vers) != 0) {
        LOGE("binder version mismatch: %s", strerror(errno));
        close(fd);
        return -1;
    }

    return fd;
}

// Send BC_TRANSACTION to create a MediaProjection
int send_create_projection_transaction(int binder_fd, uint32_t handle) {
    struct {
        uint32_t strict_mode;
        uint32_t interface_token_len;
        char interface_token[128];
        uint32_t param1;
        uint32_t param2_len;
        char param2[128];
        uint32_t param3;
        uint32_t param4;
    } __attribute__((packed)) data;

    memset(&data, 0, sizeof(data));
    data.strict_mode = 0x01000000;
    const char *descriptor = "android.media.projection.IMediaProjectionManager";
    data.interface_token_len = strlen(descriptor) + 1;
    strcpy(data.interface_token, descriptor);
    data.param1 = getuid();
    const char *pkg = "com.mirrox.server";
    data.param2_len = strlen(pkg) + 1;
    strcpy(data.param2, pkg);
    data.param3 = 0;
    data.param4 = 0;

    struct binder_transaction_data txn = {
        .target = handle,
        .code = 1,
        .flags = 0x00,
        .data = {
            .ptr = {
                .ptr = (uintptr_t)&data,
                .length = sizeof(data),
            },
        },
        .data_size = sizeof(data),
        .offsets_size = 0,
        .data_buffer = 0,
    };

    struct {
        uint32_t cmd;
        struct binder_transaction_data txn;
    } __attribute__((packed)) write_buf;

    write_buf.cmd = BC_TRANSACTION;
    write_buf.txn = txn;

    ssize_t w = write(binder_fd, &write_buf, sizeof(write_buf));
    if (w < 0) {
        LOGE("write BC_TRANSACTION failed: %s", strerror(errno));
        return -1;
    }

    return 0;
}

// Receive the reply from the binder driver
jobject receive_media_projection_reply(int binder_fd, JNIEnv *env) {
    uint8_t buffer[1024];
    ssize_t r = read(binder_fd, buffer, sizeof(buffer));
    if (r < 0) {
        LOGE("read BR_REPLY failed: %s", strerror(errno));
        return NULL;
    }

    struct binder_transaction_data *txn_reply = NULL;
    size_t pos = 0;

    while (pos + sizeof(uint32_t) < r) {
        uint32_t cmd = *(uint32_t *)(buffer + pos);
        pos += sizeof(uint32_t);

        if (cmd == BR_TRANSACTION) {
            if (pos + sizeof(struct binder_transaction_data) <= r) {
                txn_reply = (struct binder_transaction_data *)(buffer + pos);
                break;
            }
        }

        pos += sizeof(struct binder_transaction_data);
    }

    if (!txn_reply) {
        LOGE("No BR_TRANSACTION in binder reply");
        return NULL;
    }

    uint8_t *data_start = (uint8_t *)txn_reply + sizeof(struct binder_transaction_data);
    int32_t binder_handle = *(int32_t *)data_start;

    if (binder_handle == 0) {
        LOGE("Binder handle is null");
        return NULL;
    }

    jclass binderClass = (*env)->FindClass(env, "android/os/Binder");
    if (!binderClass) {
        LOGE("Failed to find android.os.Binder class");
        return NULL;
    }

    jmethodID getBinderFromHandle = (*env)->GetStaticMethodID(env, binderClass, "getBinderFromHandle", "(I)Landroid/os/IBinder;");
    if (!getBinderFromHandle) {
        LOGE("Failed to find getBinderFromHandle method");
        return NULL;
    }

    jobject result = (*env)->CallStaticObjectMethod(env, binderClass, getBinderFromHandle, binder_handle);
    return result;
}
