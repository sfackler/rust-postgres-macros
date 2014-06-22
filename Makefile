OBJS := $(shell find postgres/src/backend -name '*.o' | \
    egrep -v '(main/main\.o|snowball|libpqwalreceiver|conversion_procs)' | \
    xargs echo)
OBJS += postgres/src/timezone/localtime.o \
	postgres/src/timezone/strftime.o \
	postgres/src/timezone/pgtz.o \
	postgres/src/common/libpgcommon_srv.a \
	postgres/src/port/libpgport_srv.a

test: $(OBJS) test.c
	gcc -O2 -I postgres/src/include -lm -ldl $^
