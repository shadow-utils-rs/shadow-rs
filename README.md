<!-- spell-checker:ignore reimplementation setuid nscd subuid subgid gshadow -->
<div align="center">

# shadow-rs

[![License](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/shadow-utils-rs/shadow-rs/blob/main/LICENSE)

</div>

---

shadow-rs is a memory-safe reimplementation of the Linux
[shadow-utils](https://github.com/shadow-maint/shadow) in
[Rust](http://www.rust-lang.org). shadow-utils (`useradd`, `passwd`,
`groupadd`, etc.) is the suite of setuid-root tools that manages user accounts,
passwords, and groups on every Linux system.

## Why

shadow-utils runs as **root or setuid-root on every Linux system**. It parses
user-supplied input, writes to `/etc/passwd`, `/etc/shadow`, `/etc/group`, and
has had recent CVEs (CVE-2023-4641: password leak in memory, CVE-2024-56433:
subuid collision enabling account takeover). There is **no Rust
reimplementation** — not in uutils, not in Prossimo/Trifecta, not on crates.io.

[sudo-rs](https://github.com/trifectatechfoundation/sudo-rs) proved the model:
an independent Rust rewrite of a privilege-boundary tool can go from zero to
default-in-Ubuntu in under 3 years. shadow-rs follows that playbook.

## Goals

- **Drop-in replacement**: same flags, same exit codes, same output format as
  GNU shadow-utils. Differences are treated as bugs.
- **uutils compatible**: built on [`uucore`](https://crates.io/crates/uucore)
  with the standard `uumain()` / `uu_app()` API contract. Designed to merge
  into the uutils ecosystem.
- **Memory safe**: eliminate entire classes of vulnerabilities (buffer overflows,
  use-after-free, uninitialized memory) that affect the C original. Passwords
  zeroed in memory via `zeroize`.
- **Well-tested**: unit tests, property-based tests (`proptest`), integration
  tests, fuzz targets for all parsers. Tested on Debian, Alpine (musl), and
  Fedora (SELinux).
- **Auditable**: small dependency tree, `cargo-deny` license and advisory
  checks, no GPL dependencies.

## Status

| Tool | Status |
|------|--------|
| `passwd` | **All 17 flags implemented.** Drop-in for GNU passwd. PAM password change, `--root`, `--quiet`, `--stdin`. Output bit-for-bit identical with GNU. |
| `pwck` | Planned (Phase 1) |
| `useradd` | Planned (Phase 2) |
| `userdel` | Planned (Phase 2) |
| `usermod` | Planned (Phase 2) |
| `chpasswd` | Planned (Phase 2) |
| `chage` | Planned (Phase 2) |
| `groupadd` | Planned (Phase 3) |
| `groupdel` | Planned (Phase 3) |
| `groupmod` | Planned (Phase 3) |
| `grpck` | Planned (Phase 3) |
| `chfn` | Planned (Phase 3) |
| `chsh` | Planned (Phase 3) |
| `newgrp` | Planned (Phase 3) |

## Building

### Requirements

- Rust (stable toolchain)
- Linux (PAM headers, SELinux headers optional)
- Docker + Docker Compose (for testing)

### Build

```shell
git clone https://github.com/shadow-utils-rs/shadow-rs
cd shadow-rs
docker compose build debian
docker compose run --rm debian cargo build --release
```

### Test

All builds and tests run inside Docker containers to isolate from the host
system. Three distros are tested to catch libc and PAM differences:

```shell
docker compose run --rm debian cargo test --workspace    # Debian Trixie (glibc)
docker compose run --rm alpine cargo test --workspace    # Alpine (musl libc)
docker compose run --rm fedora cargo test --workspace    # Fedora (SELinux enforcing)
```

### Lint

```shell
docker compose run --rm debian cargo clippy --workspace --all-targets -- -D warnings
docker compose run --rm debian cargo fmt --all --check
```

## Architecture

Cargo workspace monorepo built on [`uucore`](https://crates.io/crates/uucore):

```
src/bin/shadow-rs.rs     multicall binary (dispatches by argv[0])
        |
src/uu/{tool}/           individual tool crates (passwd, useradd, ...)
        |
   ┌────┴────┐
uucore    shadow-core    shared infrastructure + domain library
```

Tools use `uucore` for the standard uutils API (`UResult`, `#[uucore::main]`,
`show_error!`) and `shadow-core` for domain-specific functionality.

**shadow-core** provides:
- File parsers for `/etc/passwd`, `/etc/shadow`, `/etc/group`, `/etc/gshadow`,
  `/etc/login.defs`, `/etc/subuid`, `/etc/subgid`
- Atomic file writes (lock, write tmp, fsync, rename, unlock, invalidate nscd)
- PAM integration (feature-gated)
- Username/groupname validation
- UID/GID allocation
- SELinux context handling (feature-gated)

Each **tool crate** exports `uumain()` and `uu_app()`, following
[uutils](https://github.com/uutils/coreutils) conventions exactly so a future
merge is frictionless.

## Docker Test Matrix

| Target | Base | libc | PAM | SELinux |
|--------|------|------|-----|---------|
| `debian` | `rust:latest` (Trixie) | glibc | Linux-PAM | headers |
| `alpine` | `rust:alpine` | musl | Linux-PAM | none |
| `fedora` | `fedora:latest` | glibc | Linux-PAM | enforcing |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Important**: shadow-rs is developed under a strict GPL clean-room policy. Do
**not** read, reference, or feed into an LLM any code from
[shadow-maint/shadow](https://github.com/shadow-maint/shadow) (GPL-2.0+).
Reference only: POSIX specs, man pages, BSD-licensed implementations (FreeBSD,
OpenBSD, musl), and sudo-rs.

## License

shadow-rs is licensed under the [MIT License](LICENSE).

GNU shadow-utils is licensed under the GPL 2.0 or later.
