#define _GNU_SOURCE

// Define and undefine just to make there is no other write interfering with the unistd.h's one
// #ifdef write
// #undef write
// #endif

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

ssize_t write(int fd, const void *buf, size_t count);

#include <errno.h>
#include <sys/uio.h>


#include "include/binder.h"
#include "include/binderfs.h"
#include "include/ioctl.h"
#include "include/types.h"
#include "include/binder_version.h"
#include "binder_utils.h"

// Manually define this if not in headers
#define BC_TRANSACTION 0x5

struct binder_transaction_data {
    binder_uintptr_t target;      // Target binder handle
    binder_uintptr_t cookie;      // Not used
    uint32_t code;                // Transaction code (1 = createProjection)
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

int open_binder() {
    int fd = syscall(SYS_openat, AT_FDCWD, BINDER_DEVICE, O_RDWR | O_CLOEXEC);
    if (fd < 0) {
        perror("open binder failed");
        return -1;
    }

    uint32_t vers = BINDER_CURRENT_PROTOCOL_VERSION;
    if (ioctl(fd, BINDER_VERSION, &vers) != 0) {
        perror("binder version mismatch");
        close(fd);
        return -1;
    }

    return fd;
}

int send_create_projection_transaction(int binder_fd, uint32_t handle) {
    struct {
        uint32_t strict_mode;
        uint32_t interface_token_len;
        char interface_token[128];
        uint32_t param1; // uid
        uint32_t param2_len;
        char param2[128]; // package name
        uint32_t param3; // flags
        uint32_t param4; // boolean
    } __attribute__((packed)) data;

    memset(&data, 0, sizeof(data));
    data.strict_mode = 0x01000000;
    const char *descriptor = "android.media.projection.IMediaProjectionManager";
    data.interface_token_len = strlen(descriptor) + 1;
    strcpy(data.interface_token, descriptor);
    data.param1 = getuid(); // calling UID
    const char *pkg = "com.mirrox.server";
    data.param2_len = strlen(pkg) + 1;
    strcpy(data.param2, pkg);
    data.param3 = 0; // flags
    data.param4 = 0; // boolean: permanentGrant = false

    struct binder_transaction_data txn = {
        .target = handle,
        .code = 1, // TRANSACTION_createProjection
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

    ssize_t w = write(binder_fd, &write_buf, (size_t)sizeof(write_buf));
    if (w < 0) {
        perror("write BC_TRANSACTION failed");
        return -1;
    }

    return 0;
}


jobject receive_media_projection_reply(int binder_fd, JNIEnv *env) {
    uint8_t buffer[1024];
    ssize_t r = read(binder_fd, buffer, sizeof(buffer));
    if (r < 0) {
        perror("read BR_REPLY failed");
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

        // Skip unknown/unsupported binder responses
        pos += sizeof(struct binder_transaction_data);
    }

    if (!txn_reply) {
        fprintf(stderr, "No BR_TRANSACTION in binder reply\n");
        return NULL;
    }

    // ⚠️ This is a simplification — we're assuming data.ptr.buffer[0] holds a handle.
    int32_t binder_handle = ((int32_t *)(uintptr_t)txn_reply->data.ptr.buffer)[0];
    if (binder_handle == 0) {
        fprintf(stderr, "Binder handle is null\n");
        return NULL;
    }

    // Convert native binder handle to Java IBinder
    jclass binderClass = (*env)->FindClass(env, "android/os/Binder");
    if (!binderClass) {
        fprintf(stderr, "Failed to find android.os.Binder\n");
        return NULL;
    }

    jmethodID getBinderFromHandle = (*env)->GetStaticMethodID(env, binderClass, "getBinderFromHandle", "(I)Landroid/os/IBinder;");
    if (!getBinderFromHandle) {
        fprintf(stderr, "Failed to find method getBinderFromHandle\n");
        return NULL;
    }

    jobject result = (*env)->CallStaticObjectMethod(env, binderClass, getBinderFromHandle, binder_handle);
    return result;
}