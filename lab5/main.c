#include <stdio.h>
#include <unistd.h>

#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/types.h>

#include <sys/epoll.h>

#include "callback.h"
#include "epoll.h"
#include "net.h"
#include "state.h"

#define MAX_EVENTS 512

int main() {
    int serverfd = net_server(1080, 10);
    if (serverfd < 0) {
        perror("creating server");
        return -1;
    }

    int dnsfd = net_dns();
    if (dnsfd < 0) {
        perror("creating dns");
        return -1;
    }

    int epfd = epoll_create(1024);
    if (epfd < 0) {
        perror("creating epoll");
        return -1;
    }

    Context context = {
        .serverfd = serverfd,
        .dnsfd = dnsfd,
        .epfd = epfd,
    };

    struct epoll_event events[MAX_EVENTS];

    Callback *server_cb = CallbackCreate(OnIncomingConnection, NULL);
    epoll_add(epfd, serverfd, EPOLLIN, server_cb);

    Callback *dns_cb = CallbackCreate(OnDNSResponse, NULL);
    epoll_add(epfd, dnsfd, EPOLLIN, dns_cb);

    while (1) {
        int num_events = epoll_wait(epfd, events, MAX_EVENTS, -1);
        if (num_events < 0) {
            perror("epoll_wait");
            return -1;
        }

        for (int i = 0; i < num_events; i++) {
            Callback *callback = events[i].data.ptr;
            callback->func(&context, callback->arg);
            CallbackDestroy(callback);
        }
    }

    close(epfd);
    close(dnsfd);
    close(serverfd);
}
