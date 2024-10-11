#include <stdlib.h>

#include "callback.h"

Callback *CallbackCreate(void *func, void *arg) {
    Callback *callback = malloc(sizeof(Callback));
    callback->func = func;
    callback->arg = arg;
    return callback;
}

void CallbackDestroy(Callback *callback) {
    free(callback);
}
