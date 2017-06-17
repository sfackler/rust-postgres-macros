VERSION = 9.4.4
SOURCE_URL = https://ftp.postgresql.org/pub/source/v$(VERSION)/postgresql-$(VERSION).tar.gz

BUILDDIR ?= $(OUT_DIR)/build
TMPDIR ?= $(OUT_DIR)/tmp
POSTGRES_DIR := $(BUILDDIR)/postgresql-$(VERSION)
CFLAGS ?= -O2 -fPIC -Wall -Wextra

POSTGRES_OBJS = $(shell find $(POSTGRES_DIR)/src/backend -name '*.o' | \
	    egrep -v '(main/main\.o|snowball|libpqwalreceiver|conversion_procs)' | \
	    xargs echo) \
	$(shell find $(POSTGRES_DIR)/src/port -name '*_srv.o') \
	$(shell find $(POSTGRES_DIR)/src/common -name '*_srv.o') \
	$(POSTGRES_DIR)/src/timezone/localtime.o \
	$(POSTGRES_DIR)/src/timezone/strftime.o \
	$(POSTGRES_DIR)/src/timezone/pgtz.o

POSTGRES_STAMP := $(BUILDDIR)/postgres.stamp
PARSER := $(BUILDDIR)/parser.o

ARCHIVE := $(OUT_DIR)/libparser.a

$(ARCHIVE): $(POSTGRES_STAMP) $(PARSER) | $(BUILDDIR)
	$(AR) -rcs $@ $(PARSER) $(POSTGRES_OBJS)

$(PARSER): src/parser.c src/parser.h $(POSTGRES_STAMP) | $(BUILDDIR)
	$(CC) $(CFLAGS) -I $(POSTGRES_DIR)/src/include -c -o $@ $<

# Postgres's build system tacks this onto CFLAGS
unexport PROFILE
$(POSTGRES_STAMP): $(POSTGRES_DIR)
	cd $(POSTGRES_DIR) && ./configure CFLAGS="$(CFLAGS)"
	$(MAKE) -C $(POSTGRES_DIR)
	touch $(POSTGRES_STAMP)

$(POSTGRES_DIR): | $(TMPDIR) $(BUILDDIR)
	curl $(SOURCE_URL) | tar xzf - -C $(TMPDIR)
	mv $(TMPDIR)/* $(BUILDDIR)

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

$(TMPDIR):
	mkdir -p $(TMPDIR)
