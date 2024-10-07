#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

#include "net.h"

int net_server(int port, int backlog) {
    int sockfd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (sockfd < 0) {
        return -1;
    }

    int true = 1;
    int err = setsockopt(sockfd, SOL_SOCKET, SO_REUSEADDR, &true, sizeof(true));
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
    int sockfd = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
    if (sockfd < 0) {
        return -1;
    }

    struct sockaddr_in dns_addr = {
        .sin_family = AF_INET,
        .sin_addr.s_addr = htonl(0x08080808),
        .sin_port = htons(53),
    };

    int err = connect(sockfd, (struct sockaddr *)&dns_addr, sizeof(dns_addr));
    if (err) {
        close(sockfd);
        return -1;
    }

    return sockfd;
}
