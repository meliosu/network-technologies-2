#include <stdio.h>
#include <sys/epoll.h>

#include "callback.h"
#include "epoll.h"

int epoll_add(int epfd, int fd, int events, Callback *callback) {
    struct epoll_event event = {
        .events = events | EPOLLONESHOT,
        .data.ptr = callback,
    };

    int err = epoll_ctl(epfd, EPOLL_CTL_ADD, fd, &event);
    if (err) {
        perror("epoll_add");
    }

    return err;
}

int epoll_mod(int epfd, int fd, int events, Callback *callback) {
    struct epoll_event event = {
        .events = events | EPOLLONESHOT,
        .data.ptr = callback,
    };

    int err = epoll_ctl(epfd, EPOLL_CTL_MOD, fd, &event);
    if (err) {
        perror("epoll_mod");
    }

    return err;
}

int epoll_del(int epfd, int fd) {
    int err = epoll_ctl(epfd, EPOLL_CTL_DEL, fd, NULL);
    if (err) {
        perror("epoll_del");
    }

    return err;
}
