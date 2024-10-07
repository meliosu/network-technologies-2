#ifndef SOCKS_STATE_H
#define SOCKS_STATE_H

typedef struct {
    int serverfd;
    int dnsfd;
    struct io_uring *ring;
} Context;

typedef struct {
    int clientfd;
    int remotefd;
    int len;
    int cap;
    void *buf;
} ClientContext;

ClientContext *ClientContextCreate(int clientfd, int cap);
void ClientContextDestroy(ClientContext *context);

void OnIncomingConnection(Context *ctx, int conn);
void OnReceivedGreeting(Context *ctx, int size, ClientContext *cctx);
void OnReceivedConnect(Context *ctx, int size, ClientContext *cctx);
void OnConnectedRemote(Context *ctx, int res, ClientContext *cctx);
void OnClientData(Context *ctx, int size, ClientContext *cctx);
void OnRemoteData(Context *ctx, int size, ClientContext *cctx);

void OnReceivedDNS(Context *ctx, int size);

#endif /* SOCKS_STATE_H */
