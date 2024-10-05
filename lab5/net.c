#include <fcntl.h>
#include <unistd.h>

#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/types.h>

#include "net.h"

int net_server(int port, int backlog) {
    int err;

    int sockfd = socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK, IPPROTO_TCP);
    if (sockfd < 0) {
        return -1;
    }

    int y = 1;
    err = setsockopt(sockfd, SOL_SOCKET, SO_REUSEADDR, &y, sizeof(y));
    if (err) {
        close(sockfd);
        return -1;
    }

    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_addr.s_addr = htonl(INADDR_ANY),
        .sin_port = htons(port),
    };

    err = bind(sockfd, (struct sockaddr *)&addr, sizeof(addr));
    if (err) {
        close(sockfd);
        return -1;
    }

    err = listen(sockfd, backlog);
    if (err) {
        close(sockfd);
        return -1;
    }

    return sockfd;
}

int net_dns() {
    int err;

    int sockfd = socket(AF_INET, SOCK_DGRAM | SOCK_NONBLOCK, IPPROTO_UDP);
    if (sockfd < 0) {
        return -1;
    }

    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_addr.s_addr = htonl(INADDR_ANY),
        .sin_port = htons(0),
    };

    err = bind(sockfd, (struct sockaddr *)&addr, sizeof(addr));
    if (err) {
        close(sockfd);
        return -1;
    }

    struct sockaddr_in dns_addr = {
        .sin_family = AF_INET,
        .sin_port = 53,
    };

    err = inet_pton(AF_INET, "8.8.8.8", &dns_addr.sin_addr);
    if (err != 1) {
        close(sockfd);
        return -1;
    }

    err = connect(sockfd, (struct sockaddr *)&dns_addr, sizeof(dns_addr));
    if (err) {
        close(sockfd);
        return -1;
    }

    return sockfd;
}

int net_set_nonblocking(int fd) {
    int status = fcntl(fd, F_GETFL, 0);
    if (status < 0) {
        return -1;
    }

    int err = fcntl(fd, F_SETFL, status | O_NONBLOCK);
    if (err) {
        return -1;
    }

    return 0;
}
