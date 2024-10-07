#ifndef SOCKS_SOCKS_H
#define SOCKS_SOCKS_H

#include <stdint.h>

#define PACKED __attribute__((packed))

#define ADDR_INET 0x01
#define ADDR_INET6 0x03
#define ADDR_DNS 0x04

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;

typedef struct {
    u8 ver;
    u8 nauth;
    u8 auth[];
} PACKED GreetingRequest;

typedef struct {
    u8 ver;
    u8 cauth;
} PACKED GreetingResponse;

typedef struct {
    u8 type;
    union {
        struct {
            u32 addr;
            u16 port;
        } PACKED ipv4;

        struct {
            char addr[16];
            u16 port;
        } PACKED ipv6;

        struct {
            u8 len;
            char nameport[];
        } PACKED dns;
    };
} PACKED Address;

typedef struct {
    u8 ver;
    u8 cmd;
    u8 rsv;
    Address addr;
} PACKED ConnectRequest;

typedef struct {
    u8 ver;
    u8 status;
    u8 rsv;
    Address addr;
} PACKED ConnectResponse;

int socks_greeting_has_auth(GreetingRequest *request, u8 auth);

#endif /* SOCKS_SOCKS_H */
