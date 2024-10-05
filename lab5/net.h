#ifndef PROXY_NET_H
#define PROXY_NET_H

int net_server(int port, int backlog);
int net_dns();
int net_set_nonblocking(int fd);

#endif /* PROXY_NET_H */
