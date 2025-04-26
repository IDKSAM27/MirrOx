#ifndef BINDER_PROTOCOL_H
#define BINDER_PROTOCOL_H

#include <stdint.h>

#define BINDER_DEVICE "/dev/binder"
#define BC_TRANSACTION 0x5
#define BR_TRANSACTION 0xC0000000

struct binder_transaction_data {
    uintptr_t target;
    uintptr_t cookie;
    uint32_t code;
    uint32_t flags;

    union {
        struct {
            uintptr_t ptr;
            size_t length;
        } ptr;
        struct {
            uint64_t handle;
            uint64_t cookie;
        } u64;
    } data;

    size_t offsets_size;
    size_t data_size;
    uintptr_t data_buffer;
};

#endif
