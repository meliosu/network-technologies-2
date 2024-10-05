#ifndef SOCKS_EPOLL_H
#define SOCKS_EPOLL_H

#include "callback.h"

int epoll_add(int epfd, int fd, int events, Callback *callback);
int epoll_mod(int epfd, int fd, int events, Callback *callback);
int epoll_del(int epfd, int fd);

#endif /* SOCKS_EPOLL_H */
