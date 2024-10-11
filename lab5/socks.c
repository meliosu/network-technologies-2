#include "socks.h"

int socks_greeting_has_auth(GreetingRequest *request, u8 auth) {
    for (int i = 0; i < request->nauth; i++) {
        if (request->auth[i] == auth) {
            return 1;
        }
    }

    return 0;
}
