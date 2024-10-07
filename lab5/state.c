#include <netinet/in.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

#include <liburing.h>

#include "callback.h"
#include "socks.h"
#include "state.h"

ClientContext *ClientContextCreate(int clientfd, int cap) {
    ClientContext *context = malloc(sizeof(ClientContext));
    context->clientfd = clientfd;
    context->remotefd = 0;
    context->cap = cap;
    context->client_buf = malloc(cap);
    context->remote_buf = malloc(cap);
    context->refcount = 1;
    return context;
}

void ClientContextDestroy(ClientContext *context) {
    context->refcount -= 1;

    if (context->refcount == 0) {
        if (context->clientfd) {
            close(context->clientfd);
        }

        if (context->remotefd) {
            close(context->remotefd);
        }

        free(context->client_buf);
        free(context->remote_buf);
        free(context);
    }
}

void OnIncomingConnection(Context *ctx, int conn) {
    if (conn < 0) {
        return;
    }

    ClientContext *cctx = ClientContextCreate(conn, 64 * 1024);

    struct io_uring_sqe *sqe;
    Callback *callback;

    callback = CallbackCreate(OnReceivedGreeting, cctx);
    sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_read(sqe, cctx->clientfd, cctx->client_buf, cctx->cap, 0);
    io_uring_sqe_set_data(sqe, callback);

    callback = CallbackCreate(OnIncomingConnection, NULL);
    sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_accept(sqe, ctx->serverfd, NULL, NULL, 0);
    io_uring_sqe_set_data(sqe, callback);

    io_uring_submit(ctx->ring);
}

void OnReceivedGreeting(Context *ctx, int size, ClientContext *cctx) {
    if (size <= 0) {
        ClientContextDestroy(cctx);
        return;
    }

    GreetingRequest *request = cctx->client_buf;
    GreetingResponse *response = cctx->remote_buf;

    if (request->ver == 0x05 && socks_greeting_has_auth(request, 0x00)) {
        response->ver = 0x05;
        response->cauth = 0x00;
    } else {
        response->ver = 0x05;
        response->cauth = 0xFF;
    }

    // TODO: make this asynchronous
    int n = write(cctx->clientfd, cctx->remote_buf, sizeof(GreetingResponse));
    if (n < 0) {
        perror("write");
    }

    if (response->cauth == 0xFF) {
        ClientContextDestroy(cctx);
        return;
    }

    Callback *callback = CallbackCreate(OnReceivedConnect, cctx);
    struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_read(sqe, cctx->clientfd, cctx->client_buf, cctx->cap, 0);
    io_uring_sqe_set_data(sqe, callback);
    io_uring_submit(ctx->ring);
}

void OnReceivedConnect(Context *ctx, int size, ClientContext *cctx) {
    if (size <= 0) {
        ClientContextDestroy(cctx);
        return;
    }

    ConnectRequest *request = cctx->client_buf;

    if (request->addr.type == ADDR_INET) {
        int sockfd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
        if (sockfd < 0) {
            ClientContextDestroy(cctx);
            return;
        }

        cctx->remotefd = sockfd;

        struct sockaddr_in addr = {
            .sin_family = AF_INET,
            .sin_addr.s_addr = request->addr.ipv4.addr,
            .sin_port = request->addr.ipv4.port,
        };

        Callback *callback = CallbackCreate(OnConnectedRemote, cctx);
        struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
        io_uring_prep_connect(sqe, sockfd, (struct sockaddr *)&addr,
                              sizeof(addr));
        io_uring_sqe_set_data(sqe, callback);
        io_uring_submit(ctx->ring);

    } else {
        printf("dns or ipv6 not supported yet\n");
        ClientContextDestroy(cctx);
        return;
    }
}

void OnConnectedRemote(Context *ctx, int res, ClientContext *cctx) {
    ConnectRequest *request = cctx->client_buf;
    ConnectResponse *response = cctx->remote_buf;

    response->ver = 0x05;
    response->rsv = 0x00;

    if (res < 0) {
        response->status = 0x01;
    } else {
        response->status = 0x00;
        response->addr = request->addr;
    }

    // TODO: make async
    // remove magic const
    int n = write(cctx->clientfd, cctx->remote_buf, 10);
    if (n < 0) {
        perror("write");
    }

    if (response->status != 0x00) {
        ClientContextDestroy(cctx);
    } else {
        cctx->refcount += 1;

        Callback *callback;
        struct io_uring_sqe *sqe;

        callback = CallbackCreate(OnClientData, cctx);
        sqe = io_uring_get_sqe(ctx->ring);
        io_uring_prep_read(sqe, cctx->clientfd, cctx->client_buf, cctx->cap, 0);
        io_uring_sqe_set_data(sqe, callback);

        callback = CallbackCreate(OnRemoteData, cctx);
        sqe = io_uring_get_sqe(ctx->ring);
        io_uring_prep_read(sqe, cctx->remotefd, cctx->remote_buf, cctx->cap, 0);
        io_uring_sqe_set_data(sqe, callback);

        io_uring_submit(ctx->ring);
    }
}

void OnClientData(Context *ctx, int size, ClientContext *cctx) {
    if (size <= 0) {
        ClientContextDestroy(cctx);
        return;
    }

    // TODO: make this asynchronous
    int n = write(cctx->remotefd, cctx->client_buf, size);
    if (n < size) {
        perror("write");
    }

    Callback *callback = CallbackCreate(OnClientData, cctx);
    struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_read(sqe, cctx->clientfd, cctx->client_buf, cctx->cap, 0);
    io_uring_sqe_set_data(sqe, callback);
    io_uring_submit(ctx->ring);
}

void OnRemoteData(Context *ctx, int size, ClientContext *cctx) {
    if (size <= 0) {
        ClientContextDestroy(cctx);
        return;
    }

    // TODO: make this asynchronous
    int n = write(cctx->clientfd, cctx->remote_buf, size);
    if (n < size) {
        perror("write");
    }

    Callback *callback = CallbackCreate(OnRemoteData, cctx);
    struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_read(sqe, cctx->remotefd, cctx->remote_buf, cctx->cap, 0);
    io_uring_sqe_set_data(sqe, callback);
    io_uring_submit(ctx->ring);
}
