PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/sbin
PROFILE ?= release

TOOLS = passwd pwck useradd userdel usermod chpasswd chage \
        groupadd groupdel groupmod grpck chfn chsh newgrp

.PHONY: all build test install uninstall clean

all: build

build:
	cargo build --profile $(PROFILE)

test:
	cargo test --workspace

install: build
	install -Dm755 target/$(PROFILE)/shadow-rs $(DESTDIR)$(BINDIR)/shadow-rs
	@for tool in $(TOOLS); do \
		ln -sf shadow-rs $(DESTDIR)$(BINDIR)/$$tool; \
	done
	@echo "Installed shadow-rs + $(words $(TOOLS)) symlinks to $(DESTDIR)$(BINDIR)/"

uninstall:
	@for tool in $(TOOLS); do \
		rm -f $(DESTDIR)$(BINDIR)/$$tool; \
	done
	rm -f $(DESTDIR)$(BINDIR)/shadow-rs
	@echo "Uninstalled shadow-rs from $(DESTDIR)$(BINDIR)/"

clean:
	cargo clean
