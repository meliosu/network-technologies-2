#include <stdio.h>
#include <string.h>

#include <liburing.h>

#include "callback.h"
#include "net.h"
#include "state.h"

int main() {
    int err;

    int server = net_server(1080, 10);
    if (server < 0) {
        perror("net_server");
        return -1;
    }

    int dns = net_dns();
    if (dns < 0) {
        perror("net_dns");
        return -1;
    }

    struct io_uring ring;

    err = io_uring_queue_init(1024, &ring, 0);
    if (err) {
        printf("io_uring_queue_init: %s\n", strerror(-err));
        return -1;
    }

    Context context = {
        .serverfd = server,
        .dnsfd = dns,
        .ring = &ring,
    };

    struct io_uring_cqe *cqe;

    // TODO: bootstrap event loop

    while (1) {
        err = io_uring_wait_cqe(&ring, &cqe);
        if (err) {
            printf("io_uring_wait_cqe: %s\n", strerror(-err));
            break;
        }

        Callback *callback = (Callback *)cqe->user_data;
        callback->func(&context, cqe->res, callback->arg);
        CallbackDestroy(callback);
    }

    io_uring_queue_exit(&ring);
}
