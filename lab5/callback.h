#ifndef SOCKS_CALLBACK_H
#define SOCKS_CALLBACK_H

#include "state.h"

typedef struct {
    void (*func)(Context *, int, void *);
    void *arg;
} Callback;

Callback *CallbackCreate(void *func, void *arg);
void CallbackDestroy(Callback *callback);

#endif /* SOCKS_CALLBACK_H */
