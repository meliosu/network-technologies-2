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

#endif /* SOCKS_STATE_H */
