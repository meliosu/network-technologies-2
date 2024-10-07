#include <stdlib.h>
#include <unistd.h>

#include "state.h"

ClientContext *ClientContextCreate(int clientfd, int cap) {
    ClientContext *context = malloc(sizeof(ClientContext));
    context->clientfd = clientfd;
    context->remotefd = 0;
    context->buf = malloc(cap);
    context->cap = cap;
    context->len = 0;
    return context;
}

void ClientContextDestroy(ClientContext *context) {
    if (context->clientfd) {
        close(context->clientfd);
    }

    if (context->remotefd) {
        close(context->remotefd);
    }

    free(context->buf);
    free(context);
}
