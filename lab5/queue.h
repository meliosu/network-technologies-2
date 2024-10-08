#ifndef SOCKS_QUEUE_H
#define SOCKS_QUEUE_H

typedef struct _Node {
    unsigned short key;
    void *value;
    struct _Node *prev;
    struct _Node *next;
} Node;

typedef struct {
    Node *first;
    Node *last;
} Queue;

void queue_insert(Queue *queue, unsigned short key, void *value);
void queue_remove(Queue *queue, unsigned short key, void **value);

#endif /* SOCKS_QUEUE_H */
