BUILDDIR ?= build
CFLAGS ?= -O2 -fPIC
RUSTC ?= rustc

POSTGRES_OBJS = $(shell find postgres/src/backend -name '*.o' | \
	    egrep -v '(main/main\.o|snowball|libpqwalreceiver|conversion_procs)' | \
	    xargs echo) \
	$(shell find postgres/src/port -name '*_srv.o') \
	postgres/src/timezone/localtime.o \
	postgres/src/timezone/strftime.o \
	postgres/src/timezone/pgtz.o \
	postgres/src/common/relpath_srv.o

POSTGRES_STAMP := $(BUILDDIR)/postgres.stamp

LIB_FILE = src/lib.rs
LIB_NAME = $(BUILDDIR)/$(shell rustc --crate-file-name $(LIB_FILE))
LIB_DEPS = $(BUILDDIR)/lib.dep

ARCHIVE = $(BUILDDIR)/libparser.a

-include $(LIB_DEPS)

all: $(LIB_NAME)

$(LIB_NAME): $(LIB_FILE) $(ARCHIVE) | $(BUILDDIR)
	$(RUSTC) -L build --out-dir $(BUILDDIR) --dep-info $(LIB_DEPS) $<

$(ARCHIVE): $(POSTGRES_STAMP) $(BUILDDIR)/parser.o | $(BUILDDIR)
	$(AR) -rcs $@ $(BUILDDIR)/parser.o $(POSTGRES_OBJS)

$(BUILDDIR)/parser.o: src/parser.c src/parser.h | $(BUILDDIR)
	$(CC) $(CFLAGS) -I postgres/src/include -c -o $@ $<

$(POSTGRES_STAMP): | $(BUILDDIR)
	cd postgres && ./configure CFLAGS="$(CFLAGS)"
	$(MAKE) -C postgres
	touch $(POSTGRES_STAMP)

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

clean:
	rm -rf $(BUILDDIR)

.PHONY: all clean
