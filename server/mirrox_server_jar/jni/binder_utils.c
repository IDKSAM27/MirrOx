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

    ssize_t w = write(binder_fd, &write_buf, sizeof(write_buf));
    if (w < 0) {
        perror("write BC_TRANSACTION failed");
        return -1;
    }

    return 0;
}
