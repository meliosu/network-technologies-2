#include <stdlib.h>

#include "queue.h"

void queue_insert(Queue *queue, unsigned short key, void *value) {
    Node *node = malloc(sizeof(Node));
    node->key = key;
    node->value = value;
    node->prev = NULL;
    node->next = NULL;

    if (!queue->first) {
        queue->first = queue->last = node;
    } else {
        node->prev = queue->last;
        queue->last = queue->last->next = node;
    }
}

void queue_remove(Queue *queue, unsigned short key, void **value) {
    Node *curr = queue->first;

    while (curr) {
        if (curr->key == key) {
            if (curr->next) {
                curr->next->prev = curr->prev;
            }

            if (curr->prev) {
                curr->prev->next = curr->next;
            }

            if (curr == queue->first) {
                queue->first = curr->next;
            }

            if (curr == queue->last) {
                queue->last = curr->prev;
            }

            *value = curr->value;
            free(curr);
            return;
        }

        curr = curr->next;
    }

    *value = NULL;
}
