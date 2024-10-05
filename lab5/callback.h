#ifndef SOCKS_CALLBACK_H
#define SOCKS_CALLBACK_H

typedef struct Context {
    int epfd;
    int serverfd;
    int dnsfd;
} Context;

typedef struct Callback {
    void (*func)(Context *, void *);
    void *arg;
} Callback;

Callback *CallbackCreate(void *func, void *arg);
void CallbackDestroy(Callback *callback);

#endif /* SOCKS_CALLBACK_H */
