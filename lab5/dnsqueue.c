#include <stdlib.h>
#include <string.h>

#include "dnsqueue.h"

void DNSQueueInsert(DNSQueue *queue, char *domain, void *context) {
    DNSNode *node = malloc(sizeof(DNSNode));
    node->domain = domain;
    node->context = context;
    node->prev = NULL;
    node->next = NULL;

    if (!queue->first) {
        queue->first = queue->last = node;
    } else {
        node->prev = queue->last;
        queue->last = queue->last->next = node;
    }
}

void DNSQueueRemove(DNSQueue *queue, char *domain, void **context) {
    DNSNode *curr = queue->first;

    while (curr) {
        if (!strcmp(domain, curr->domain)) {
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

            *context = curr->context;
            free(curr);
            return;
        }

        curr = curr->next;
    }

    *context = NULL;
}
