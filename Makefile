PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/sbin

# Tools that need setuid-root to allow non-root callers (change own password,
# GECOS, shell, effective group).
SETUID_TOOLS = passwd chfn chsh newgrp

# Root-only tools (no setuid; fail at getuid() check for non-root callers).
ROOT_TOOLS = useradd userdel usermod chpasswd chage \
             groupadd groupdel groupmod pwck grpck

ALL_TOOLS = $(SETUID_TOOLS) $(ROOT_TOOLS)

.PHONY: all build build-multicall test install install-multicall uninstall clean

all: build

build:
	cargo build --release --workspace --bins --exclude shadow-rs

build-multicall:
	cargo build --release --bin shadow-rs

test:
	cargo test --workspace

# Default install: 14 standalone per-tool binaries with least-privilege setuid
# layout matching GNU shadow-utils. Only passwd/chfn/chsh/newgrp are setuid.
install: build
	@for tool in $(SETUID_TOOLS); do \
		install -Dm4755 target/release/$$tool $(DESTDIR)$(BINDIR)/$$tool || exit 1; \
	done
	@for tool in $(ROOT_TOOLS); do \
		install -Dm0755 target/release/$$tool $(DESTDIR)$(BINDIR)/$$tool || exit 1; \
	done
	@echo "Installed $(words $(ALL_TOOLS)) standalone binaries to $(DESTDIR)$(BINDIR)/"
	@echo "  setuid (4755): $(SETUID_TOOLS)"
	@echo "  root-only (0755): $(ROOT_TOOLS)"

# Opt-in install: single multicall binary with symlinks. Smaller footprint but
# larger setuid attack surface — the binary is installed setuid-root, so all 14
# applets run with euid=root when invoked via symlink. Intended for
# container/embedded use where disk savings matter and attack surface does not.
install-multicall: build-multicall
	install -Dm4755 target/release/shadow-rs $(DESTDIR)$(BINDIR)/shadow-rs
	@for tool in $(ALL_TOOLS); do \
		ln -sf shadow-rs $(DESTDIR)$(BINDIR)/$$tool; \
	done
	@echo "Installed multicall shadow-rs + $(words $(ALL_TOOLS)) symlinks to $(DESTDIR)$(BINDIR)/"

uninstall:
	@for tool in $(ALL_TOOLS); do \
		rm -f $(DESTDIR)$(BINDIR)/$$tool; \
	done
	rm -f $(DESTDIR)$(BINDIR)/shadow-rs
	@echo "Uninstalled shadow-rs from $(DESTDIR)$(BINDIR)/"

clean:
	cargo clean
