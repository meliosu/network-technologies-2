#ifndef SOCKS_DNSQUEUE_H
#define SOCKS_DNSQUEUE_H

typedef struct DNSNode {
    char *domain;
    void *context;
    struct DNSNode *prev;
    struct DNSNode *next;
} DNSNode;

typedef struct DNSQueue {
    DNSNode *first;
    DNSNode *last;
} DNSQueue;

void DNSQueueInsert(DNSQueue *queue, char *domain, void *context);
void DNSQueueRemove(DNSQueue *queue, char *domain, void **context);

#endif
