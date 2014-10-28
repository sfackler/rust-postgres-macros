BUILDDIR ?= build
CFLAGS ?= -O2 -fPIC -Wall -Wextra

POSTGRES_OBJS = $(shell find postgres/src/backend -name '*.o' | \
	    egrep -v '(main/main\.o|snowball|libpqwalreceiver|conversion_procs)' | \
	    xargs echo) \
	$(shell find postgres/src/port -name '*_srv.o') \
	postgres/src/timezone/localtime.o \
	postgres/src/timezone/strftime.o \
	postgres/src/timezone/pgtz.o \
	postgres/src/common/relpath_srv.o

POSTGRES_STAMP := $(BUILDDIR)/postgres.stamp
PARSER := $(BUILDDIR)/parser.o

ARCHIVE := $(OUT_DIR)/libparser.a

$(ARCHIVE): $(POSTGRES_STAMP) $(PARSER) | $(BUILDDIR)
	$(AR) -rcs $@ $(PARSER) $(POSTGRES_OBJS)

$(PARSER): src/parser.c src/parser.h | $(BUILDDIR)
	$(CC) $(CFLAGS) -I postgres/src/include -c -o $@ $<

# Postgres's build system tacks this onto CFLAGS
unexport PROFILE
$(POSTGRES_STAMP): | $(BUILDDIR)
	cd postgres && ./configure CFLAGS="$(CFLAGS)"
	$(MAKE) -C postgres
	touch $(POSTGRES_STAMP)

$(BUILDDIR):
	mkdir -p $(BUILDDIR)
