BUILDDIR ?= build
DEPS_DIR ?= $(BUILDDIR)
CFLAGS ?= -O2 -fPIC -Wall -Wextra
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
LIB_NAME = $(BUILDDIR)/$(shell rustc --print-file-name $(LIB_FILE))
LIB_DEPS = $(BUILDDIR)/lib.dep

ARCHIVE = $(DEPS_DIR)/libparser.a

-include $(LIB_DEPS)

all: $(LIB_NAME)

archive: $(ARCHIVE)

$(LIB_NAME): $(LIB_FILE) $(ARCHIVE) | $(BUILDDIR)
	$(RUSTC) -L build --out-dir $(BUILDDIR) --dep-info $(LIB_DEPS) $<

$(ARCHIVE): $(POSTGRES_STAMP) $(DEPS_DIR)/parser.o | $(BUILDDIR)
	$(AR) -rcs $@ $(DEPS_DIR)/parser.o $(POSTGRES_OBJS)

$(DEPS_DIR)/parser.o: src/parser.c src/parser.h | $(BUILDDIR)
	$(CC) $(CFLAGS) -I postgres/src/include -c -o $@ $<

$(POSTGRES_STAMP): | $(BUILDDIR)
	cd postgres && ./configure CFLAGS="$(CFLAGS)"
	$(MAKE) -C postgres
	touch $(POSTGRES_STAMP)

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

clean:
	rm -rf $(BUILDDIR)

.PHONY: all archive clean
