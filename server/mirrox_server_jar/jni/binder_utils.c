#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/ioctl.h>
#include <string.h>
#include "include/binder.h"
#include "include/binderfs.h"
#include "include/ioctl.h"
#include "include/types.h"
#include "binder_utils.h"

int open_binder() {
    int fd = open(BINDER_DEVICE, O_RDWR | O_CLOEXEC);
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
