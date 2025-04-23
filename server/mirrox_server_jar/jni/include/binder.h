#ifndef MIRROX_BINDER_H
#define MIRROX_BINDER_H

#include <linux/ioctl.h>
#include <stdint.h>

#define BINDER_CURRENT_PROTOCOL_VERSION 8
#define BINDER_TYPE_BINDER 0x85
#define BINDER_TYPE_HANDLE 0x86
#define BINDER_WRITE_READ 40000930U
#define BINDER_VERSION 0x40046208
#define BINDER_SET_MAX_THREADS 0x40046205
#define BINDER_THREAD_EXIT 0x40046206

#define BINDER_DEVICE "/dev/binder"

struct binder_write_read {
    uint64_t write_size;
    uint64_t write_consumed;
    uint64_t write_buffer;

    uint64_t read_size;
    uint64_t read_consumed;
    uint64_t read_buffer;
};

#endif // MIRROX_BINDER_H
