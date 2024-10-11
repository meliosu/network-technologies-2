#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <liburing.h>

#include "callback.h"
#include "net.h"
#include "state.h"

unsigned short get_port(int argc, char **argv) {
    if (argc < 2) {
        return 1080;
    }

    int port = atoi(argv[1]);
    if (port == 0) {
        return 1080;
    }

    return port;
}

int main(int argc, char **argv) {
    int err;

    printf("[%d]\n", getpid());

    signal(SIGPIPE, SIG_IGN);

    int port = get_port(argc, argv);

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

    Context ctx = {
        .serverfd = server,
        .dnsfd = dns,
        .ring = &ring,
        .questions.first = NULL,
        .questions.last = NULL,
        .dns_buf = malloc(64 * 1024),
        .dns_buflen = 64 * 1024,
    };

    Callback *callback = CallbackCreate(OnIncomingConnection, NULL);
    struct io_uring_sqe *sqe = io_uring_get_sqe(ctx.ring);
    io_uring_prep_accept(sqe, ctx.serverfd, NULL, NULL, 0);
    io_uring_sqe_set_data(sqe, callback);
    io_uring_submit(ctx.ring);

    struct io_uring_cqe *cqe;

    while (1) {
        err = io_uring_wait_cqe(&ring, &cqe);
        if (err) {
            printf("io_uring_wait_cqe: %s\n", strerror(-err));
            break;
        }

        Callback *callback = (Callback *)cqe->user_data;
        callback->func(&ctx, cqe->res, callback->arg);
        CallbackDestroy(callback);
        io_uring_cqe_seen(ctx.ring, cqe);
    }

    io_uring_queue_exit(&ring);
}
