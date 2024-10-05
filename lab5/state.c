#include <errno.h>
#include <netinet/in.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

#include <sys/epoll.h>

#include "callback.h"
#include "net.h"
#include "socks.h"
#include "state.h"

int epoll_add(int epfd, int fd, int events, Callback *callback) {
    struct epoll_event event = {
        .events = events | EPOLLONESHOT,
        .data.ptr = callback,
    };

    int err = epoll_ctl(epfd, EPOLL_CTL_ADD, fd, &event);
    if (err) {
        perror("epoll_add");
    }

    return err;
}

int epoll_mod(int epfd, int fd, int events, Callback *callback) {
    struct epoll_event event = {
        .events = events | EPOLLONESHOT,
        .data.ptr = callback,
    };

    int err = epoll_ctl(epfd, EPOLL_CTL_MOD, fd, &event);
    if (err) {
        perror("epoll_mod");
    }

    return err;
}

int epoll_del(int epfd, int fd) {
    int err = epoll_ctl(epfd, EPOLL_CTL_DEL, fd, NULL);
    if (err) {
        perror("epoll_del");
    }

    return err;
}

ClientContext *ClientContextCreate(int clientfd, int cap) {
    ClientContext *context = malloc(sizeof(ClientContext));
    context->client = clientfd;
    context->remote = 0;
    context->cap = cap;
    context->buff = malloc(cap);
    context->len = 0;
    return context;
}

void ClientContextDestroy(ClientContext *context) {
    if (context->client) {
        close(context->client);
    }

    if (context->remote) {
        close(context->remote);
    }

    free(context->buff);
}

void OnDNSResponse(Context *ctx) {}

// DONE
void OnIncomingConnection(Context *ctx) {
    int conn = accept(ctx->serverfd, NULL, NULL);
    if (conn < 0) {
        perror("accept");
    }

    int err = net_set_nonblocking(conn);
    if (err < 0) {
        perror("nonblocking");
    }

    ClientContext *context = ClientContextCreate(conn, 4096);
    Callback *callback = CallbackCreate(OnGreetingRequest, context);
    epoll_add(ctx->epfd, conn, EPOLLIN, callback);
}

// DONE
void OnGreetingRequest(Context *ctx, ClientContext *cctx) {
    int n = read(cctx->client, cctx->buff, cctx->cap);
    if (n < 0) {
        ClientContextDestroy(cctx);
        return;
    }

    GreetingRequest *request = cctx->buff;

    int has_no_auth = 0;
    for (int i = 0; i < request->nauth; i++) {
        if (request->auth[i] == 0x00) {
            has_no_auth = 1;
            break;
        }
    }

    GreetingResponse response = {
        .ver = 0x5,
    };

    if (request->ver == 0x5 && has_no_auth) {
        response.cauth = 0x00;
    } else {
        response.cauth = 0xFF;
    }

    int m = write(cctx->client, &response, sizeof(response));
    if (m < 0) {
        ClientContextDestroy(cctx);
        return;
    }

    Callback *callback = CallbackCreate(OnConnectionRequest, cctx);
    epoll_add(ctx->epfd, cctx->client, EPOLLIN, callback);
}

void OnConnectionRequest(Context *ctx, ClientContext *cctx) {
    int n = read(cctx->client, cctx->buff, cctx->cap);
    if (n < 0) {
        ClientContextDestroy(cctx);
        return;
    }

    ConnectRequest *request = cctx->buff;

    if (request->addr.type == ADDR_INET) {
        struct sockaddr_in addr = {
            .sin_family = AF_INET,
            .sin_addr.s_addr = request->addr.ipv4.addr,
            .sin_port = request->addr.ipv4.port,
        };

        int remote = socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK, IPPROTO_TCP);
        if (remote < 0) {
            perror("socket");
        }

        int err = connect(remote, (struct sockaddr *)&addr, sizeof(addr));
        if (err && errno != EINPROGRESS) {
            perror("connect");
        }

        ConnectResponse response = {
            .ver = 0x05,
            .status = 0x00,
        };

        response.addr.type = ADDR_INET;
        response.addr.ipv4.addr = request->addr.ipv4.addr;
        response.addr.ipv4.port = request->addr.ipv4.port;

        int n = write(cctx->client, &response, sizeof(response));
        if (n < 0) {
            perror("write");
            ClientContextDestroy(cctx);
            return;
        }

        cctx->remote = remote;

        epoll_mod(ctx->epfd, cctx->client, EPOLLIN,
                  CallbackCreate(OnClientData, cctx));
        epoll_add(ctx->epfd, remote, EPOLLIN,
                  CallbackCreate(OnServerData, cctx));
    } else if (request->addr.type == ADDR_INET6) {
        struct sockaddr_in6 addr = {
            .sin6_family = AF_INET6,
            .sin6_port = request->addr.ipv6.port,
        };

        memcpy(&addr.sin6_addr, request->addr.ipv6.addr, 16);

        int remote = socket(AF_INET6, SOCK_STREAM | SOCK_NONBLOCK, IPPROTO_TCP);
        if (remote < 0) {
            perror("socket");
        }

        int err = connect(remote, (struct sockaddr *)&addr, sizeof(addr));
        if (err && errno != EINPROGRESS) {
            perror("connect");
        }

        ConnectResponse response = {
            .ver = 0x05,
            .status = 0x00,
        };

        response.addr.type = ADDR_INET6;
        response.addr.ipv6.port = request->addr.ipv6.port;
        memcpy(&response.addr.ipv6, &request->addr.ipv6, 16);

        int n = write(cctx->client, &response, sizeof(response));
        if (n < 0) {
            perror("write");
            ClientContextDestroy(cctx);
            return;
        }

        cctx->remote = remote;

        epoll_mod(ctx->epfd, cctx->client, EPOLLIN,
                  CallbackCreate(OnClientData, cctx));
        epoll_add(ctx->epfd, remote, EPOLLIN,
                  CallbackCreate(OnServerData, cctx));
    } else if (request->addr.type == ADDR_DNS) {
        // TODO
    } else {
        printf("encountered unknown address type\n");
        ClientContextDestroy(cctx);
        return;
    }
}

// DONE
void OnServerData(Context *ctx, ClientContext *cctx) {
    int n = read(cctx->remote, cctx->buff, cctx->cap);
    if (n < 0) {
        ClientContextDestroy(cctx);
        return;
    }

    int m = write(cctx->client, cctx->buff, n);
    if (m < 0) {
        ClientContextDestroy(cctx);
        return;
    }

    Callback *callback = CallbackCreate(OnServerData, cctx);
    epoll_mod(ctx->epfd, cctx->remote, EPOLLIN, callback);
}

// DONE
void OnClientData(Context *ctx, ClientContext *cctx) {
    int n = read(cctx->client, cctx->buff, cctx->cap);
    if (n < 0) {
        ClientContextDestroy(cctx);
        return;
    }

    int m = write(cctx->remote, cctx->buff, n);
    if (m < 0) {
        ClientContextDestroy(cctx);
        return;
    }

    Callback *callback = CallbackCreate(OnClientData, cctx);
    epoll_mod(ctx->epfd, cctx->client, EPOLLIN, callback);
}
