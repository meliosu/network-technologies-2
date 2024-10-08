#ifndef SOCKS_STATE_H
#define SOCKS_STATE_H

#include "queue.h"

typedef struct {
    int serverfd;
    int dnsfd;
    Queue questions;
    struct io_uring *ring;
    void *dns_buf;
    int dns_buflen;
} Context;

typedef struct {
    int clientfd;
    int remotefd;
    int cap;
    void *client_buf;
    void *remote_buf;
    int refcount;
} ClientContext;

ClientContext *ClientContextCreate(int clientfd, int cap);
void ClientContextDestroy(ClientContext *context);

void OnIncomingConnection(Context *ctx, int conn);

void OnReceivedGreeting(Context *ctx, int size, ClientContext *cctx);
void OnSentGreeting(Context *ctx, int size, ClientContext *cctx);

void OnReceivedConnect(Context *ctx, int size, ClientContext *cctx);
void OnSentConnect(Context *ctx, int size, ClientContext *cctx);

void OnConnectedRemote(Context *ctx, int res, ClientContext *cctx);

void OnRcvdClientData(Context *ctx, int size, ClientContext *cctx);
void OnSentClientData(Context *ctx, int size, ClientContext *cctx);
void OnRcvdRemoteData(Context *ctx, int size, ClientContext *cctx);
void OnSentRemoteData(Context *ctx, int size, ClientContext *cctx);

void OnReceivedDNS(Context *ctx, int size);
void OnSentDNS(Context *ctx, int size, unsigned char *buffer);

#endif /* SOCKS_STATE_H */
