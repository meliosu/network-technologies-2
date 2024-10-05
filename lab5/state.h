#ifndef SOCKS_STATE_H
#define SOCKS_STATE_H

#include "callback.h"
#include "dnsqueue.h"

typedef struct ClientContext {
    int client;
    int remote;
    void *buff;
    int len;
    int cap;
} ClientContext;

ClientContext *ClientContextCreate(int clientfd, int cap);
void ClientContextDestroy(Context *ctx, ClientContext *cctx);

void OnDNSResponse(Context *ctx);
void OnIncomingConnection(Context *ctx);
void OnGreetingRequest(Context *ctx, ClientContext *cctx);
void OnConnectionRequest(Context *ctx, ClientContext *cctx);
void OnServerData(Context *ctx, ClientContext *cctx);
void OnClientData(Context *ctx, ClientContext *cctx);

#endif /* SOCKS_STATE_H */
