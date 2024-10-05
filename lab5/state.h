#ifndef SOCKS_STATE_H
#define SOCKS_STATE_H

#include "callback.h"

typedef struct ClientContext {
    int client;
    int remote;
    void *buff;
} ClientContext;

ClientContext *ClientContextCreate();
void ClientContextDestroy(ClientContext *context);

void OnDNSReadyRequest(Context *ctx, char *domain);
void OnDNSResponse(Context *ctx, ClientContext *cctx);
void OnIncomingConnection(Context *ctx);
void OnGreetingRequest(Context *ctx, ClientContext *cctx);
void OnReadyGreetingResponse(Context *ctx, ClientContext *cctx);
void OnConnectionRequest(Context *ctx, ClientContext *cctx);
void OnReadyConnectionResponse(Context *ctx, ClientContext *cctx);
void OnServerData(Context *ctx, ClientContext *cctx);
void OnClientData(Context *ctx, ClientContext *cctx);

#endif /* SOCKS_STATE_H */
