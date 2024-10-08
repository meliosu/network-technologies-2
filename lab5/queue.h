#ifndef SOCKS_QUEUE_H
#define SOCKS_QUEUE_H

typedef struct _Node {
    char *key;
    void *value;
    struct _Node *prev;
    struct _Node *next;
} Node;

typedef struct {
    Node *first;
    Node *last;
} Queue;

#endif /* SOCKS_QUEUE_H */
