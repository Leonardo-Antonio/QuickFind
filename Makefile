PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
DATADIR = $(PREFIX)/share
APPDIR = $(DATADIR)/applications

BINARY = target/release/quickfind
FEATURES ?= 

.PHONY: all build install uninstall clean

all: build

build:
	@if [ -n "$(FEATURES)" ]; then \
		cargo build --release --features "$(FEATURES)"; \
	else \
		cargo build --release; \
	fi

install: build
	install -Dm755 $(BINARY) $(DESTDIR)$(BINDIR)/quickfind
	install -Dm644 quickfind.desktop $(DESTDIR)$(APPDIR)/quickfind.desktop

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/quickfind
	rm -f $(DESTDIR)$(APPDIR)/quickfind.desktop

clean:
	cargo clean
