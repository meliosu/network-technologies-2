#ifndef SOCKS_PANIC_H
#define SOCKS_PANIC_H

#define panic(fmt, args...)                                                    \
    do {                                                                       \
        printf(fmt "\n", ##args);                                              \
        exit(-1);                                                              \
    } while (0)

#endif /* SOCKS_PANIC_H */
