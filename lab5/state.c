#include <arpa/inet.h>
#include <arpa/nameser.h>
#include <netdb.h>
#include <netinet/in.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

#include <liburing.h>

#include <ares.h>

#include "callback.h"
#include "queue.h"
#include "socks.h"
#include "state.h"

static unsigned short ID = 1;

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

void OnReceivedDNS(Context *ctx, int size) {
    if (size <= 0) {
        return;
    }

    int err;
    int id;
    struct hostent *hostent;
    ares_dns_record_t *record;
    ClientContext *cctx;

    err = ares_dns_parse(ctx->dns_buf, size, 0, &record);
    if (err != ARES_SUCCESS) {
        return;
    }

    id = ares_dns_record_get_id(record);
    queue_remove(&ctx->questions, id, (void **)&cctx);

    if (!cctx) {
        ares_dns_record_destroy(record);
        return;
    }

    err = ares_parse_a_reply(ctx->dns_buf, size, &hostent, NULL, NULL);
    if (err != ARES_SUCCESS) {
        ares_dns_record_destroy(record);

        ConnectResponse *response = cctx->remote_buf;

        response->ver = 0x05;
        response->rsv = 0x00;
        response->status = 0x01;

        Callback *callback = CallbackCreate(OnSentConnect, cctx);
        struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
        io_uring_prep_write(sqe, cctx->clientfd, cctx->remote_buf,
                            sizeof(*response), 0);
        io_uring_sqe_set_data(sqe, callback);
        io_uring_submit(ctx->ring);
        return;
    }

    ConnectRequest *request = cctx->client_buf;

    struct in_addr *addr = (struct in_addr *)hostent->h_addr_list[0];
    unsigned short port =
        *(unsigned short *)(request->addr.dns.nameport + request->addr.dns.len);

    struct sockaddr_in remote = {
        .sin_family = AF_INET,
        .sin_addr = *addr,
        .sin_port = port,
    };

    ares_free_hostent(hostent);
    ares_dns_record_destroy(record);

    int sockfd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (sockfd < 0) {
        ClientContextDestroy(cctx);
        return;
    }

    cctx->remotefd = sockfd;

    Callback *callback = CallbackCreate(OnConnectedRemote, cctx);
    struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_connect(sqe, sockfd, (struct sockaddr *)&remote,
                          sizeof(remote));
    io_uring_sqe_set_data(sqe, callback);
    io_uring_submit(ctx->ring);
}

void OnSentDNS(Context *ctx, int size, unsigned char *buffer) {
    if (size <= 0) {
        return;
    }

    ares_free_string(buffer);

    Callback *callback = CallbackCreate(OnReceivedDNS, NULL);
    struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_read(sqe, ctx->dnsfd, ctx->dns_buf, ctx->dns_buflen, 0);
    io_uring_sqe_set_data(sqe, callback);
    io_uring_submit(ctx->ring);
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

    response->ver = 0x05;

    if (request->ver == 0x05 && socks_greeting_has_auth(request, 0x00)) {
        response->cauth = 0x00;
    } else {
        response->cauth = 0xFF;
    }

    Callback *callback = CallbackCreate(OnSentGreeting, cctx);
    struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_write(sqe, cctx->clientfd, cctx->remote_buf,
                        sizeof(GreetingResponse), 0);
    io_uring_sqe_set_data(sqe, callback);
    io_uring_submit(ctx->ring);
}

void OnSentGreeting(Context *ctx, int size, ClientContext *cctx) {
    if (size <= 0) {
        ClientContextDestroy(cctx);
        return;
    }

    GreetingResponse *response = cctx->remote_buf;

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
    } else if (request->addr.type == ADDR_INET6) {
        int sockfd = socket(AF_INET6, SOCK_STREAM, IPPROTO_TCP);
        if (sockfd < 0) {
            ClientContextDestroy(cctx);
            return;
        }

        cctx->remotefd = sockfd;

        struct sockaddr_in6 addr;
        addr.sin6_family = AF_INET6;
        addr.sin6_port = request->addr.ipv6.port;
        memcpy(&addr.sin6_addr, request->addr.ipv6.addr, 16);

        Callback *callback = CallbackCreate(OnConnectedRemote, cctx);
        struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
        io_uring_prep_connect(sqe, cctx->remotefd, (struct sockaddr *)&addr,
                              sizeof(addr));
        io_uring_sqe_set_data(sqe, callback);
        io_uring_submit(ctx->ring);
    } else {
        unsigned short id = ID++;
        char *name = strndup(request->addr.dns.nameport, request->addr.dns.len);

        unsigned char *buffer;
        int buffer_len;

        ares_create_query(name, ARES_CLASS_IN, ARES_REC_TYPE_A, id, 1, &buffer,
                          &buffer_len, 0);

        free(name);

        queue_insert(&ctx->questions, id, cctx);

        Callback *callback = CallbackCreate(OnSentDNS, buffer);
        struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
        io_uring_prep_write(sqe, ctx->dnsfd, buffer, buffer_len, 0);
        io_uring_sqe_set_data(sqe, callback);
        io_uring_submit(ctx->ring);
    }
}

void OnSentConnect(Context *ctx, int size, ClientContext *cctx) {
    if (size <= 0) {
        ClientContextDestroy(cctx);
        return;
    }

    ConnectResponse *response = cctx->remote_buf;

    if (response->status != 0x00) {
        ClientContextDestroy(cctx);
        return;
    }

    cctx->refcount += 1;

    Callback *callback;
    struct io_uring_sqe *sqe;

    callback = CallbackCreate(OnRcvdClientData, cctx);
    sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_read(sqe, cctx->clientfd, cctx->client_buf, cctx->cap, 0);
    io_uring_sqe_set_data(sqe, callback);

    callback = CallbackCreate(OnRcvdRemoteData, cctx);
    sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_read(sqe, cctx->remotefd, cctx->remote_buf, cctx->cap, 0);
    io_uring_sqe_set_data(sqe, callback);

    io_uring_submit(ctx->ring);
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

    Callback *callback = CallbackCreate(OnSentConnect, cctx);
    struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_write(sqe, cctx->clientfd, cctx->remote_buf, 10, 0);
    io_uring_sqe_set_data(sqe, callback);
    io_uring_submit(ctx->ring);
}

void OnRcvdClientData(Context *ctx, int size, ClientContext *cctx) {
    if (size <= 0) {
        ClientContextDestroy(cctx);
        return;
    }

    Callback *callback = CallbackCreate(OnSentClientData, cctx);
    struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_write(sqe, cctx->remotefd, cctx->client_buf, size, 0);
    io_uring_sqe_set_data(sqe, callback);
    io_uring_submit(ctx->ring);
}

void OnSentClientData(Context *ctx, int size, ClientContext *cctx) {
    if (size <= 0) {
        ClientContextDestroy(cctx);
        return;
    }

    Callback *callback = CallbackCreate(OnRcvdClientData, cctx);
    struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_read(sqe, cctx->clientfd, cctx->client_buf, cctx->cap, 0);
    io_uring_sqe_set_data(sqe, callback);
    io_uring_submit(ctx->ring);
}

void OnRcvdRemoteData(Context *ctx, int size, ClientContext *cctx) {
    if (size <= 0) {
        ClientContextDestroy(cctx);
        return;
    }

    Callback *callback = CallbackCreate(OnSentRemoteData, cctx);
    struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_write(sqe, cctx->clientfd, cctx->remote_buf, size, 0);
    io_uring_sqe_set_data(sqe, callback);
    io_uring_submit(ctx->ring);
}

void OnSentRemoteData(Context *ctx, int size, ClientContext *cctx) {
    if (size <= 0) {
        ClientContextDestroy(cctx);
        return;
    }

    Callback *callback = CallbackCreate(OnRcvdRemoteData, cctx);
    struct io_uring_sqe *sqe = io_uring_get_sqe(ctx->ring);
    io_uring_prep_read(sqe, cctx->remotefd, cctx->remote_buf, cctx->cap, 0);
    io_uring_sqe_set_data(sqe, callback);
    io_uring_submit(ctx->ring);
}
