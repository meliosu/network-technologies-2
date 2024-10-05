CC=gcc
FLAGS=-Wall -g

OBJECTS=main.o net.o socks.o state.o dnsqueue.o dns.o callback.o

all: main

main: $(OBJECTS)
	$(CC) $(FLAGS) -o proxy $(OBJECTS)

main.o:
	$(CC) $(FLAGS) -c main.c

net.o:
	$(CC) $(FLAGS) -c net.c

socks.o:
	$(CC) $(FLAGS) -c socks.c

callback.o:
	$(CC) $(FLAGS) -c callback.c

state.o:
	$(CC) $(FLAGS) -c state.c

dnsqueue.o:
	$(CC) $(FLAGS) -c dnsqueue.c

dns.o:
	$(CC) $(FLAGS) -c dns.c


clean:
	rm *.o proxy
