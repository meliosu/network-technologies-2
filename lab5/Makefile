CC=gcc
FLAGS=-Wno-deprecated-declarations -g
TARGET=proxy
LIBS=-luring -lcares

OBJECTS=main.o callback.o net.o socks.o state.o queue.o

all: $(TARGET)

$(TARGET): $(OBJECTS)
	$(CC) $(FLAGS) -o $(TARGET) $(OBJECTS) $(LIBS)

main.o:
	$(CC) $(FLAGS) -c main.c

callback.o:
	$(CC) $(FLAGS) -c callback.c

net.o:
	$(CC) $(FLAGS) -c net.c

socks.o:
	$(CC) $(FLAGS) -c socks.c

state.o:
	$(CC) $(FLAGS) -c state.c

queue.o:
	$(CC) $(FLAGS) -c queue.c


clean:
	rm $(TARGET) *.o
