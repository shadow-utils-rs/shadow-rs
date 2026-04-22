# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**shadow-rs** is a memory-safe Rust reimplementation of Linux shadow-utils (`useradd`, `passwd`, `groupadd`, etc.) — the setuid-root tools that manage user accounts, passwords, and groups on every Linux system. Designed as a drop-in for the [uutils](https://github.com/uutils) ecosystem using `uucore`.

Status: **Active development** — `passwd` is feature-complete (all 17 flags). Full strategy and phasing in `PLAN-shadow-rs.md`.

## Key Documents

- `PLAN-shadow-rs.md` — project strategy, phasing, outreach timeline, risk matrix
- `CONTRIBUTING.md` — contributor guide, clean-room policy, PR process
- `SECURITY.md` — vulnerability reporting
- `deny.toml` — cargo-deny license allowlist and advisory config
- `docs/OPENBSD-REFERENCE.md` — security patterns from OpenBSD's passwd (ISC license)
- `docs/SECURITY-HARDENING.md` — hardening roadmap (signals, privdrop, landlock, env sanitization)

## Credits & Safe Reference Sources

This project is developed under a strict GPL clean-room policy. All implementation
work references ONLY these permissively licensed sources:

| Source | License | What we use it for |
|--------|---------|-------------------|
| [POSIX specification](https://pubs.opengroup.org/onlinepubs/9799919799/) | Open standard | Behavioral spec for each command |
| Man pages (man7.org) | Documentation | Command options, file formats, semantics |
| [FreeBSD src](https://cgit.freebsd.org/src/) | BSD-2-Clause | Reference implementation patterns |
| [OpenBSD src](https://cvsweb.openbsd.org/src/) | ISC | Security hardening patterns (pledge, unveil, privdrop) |
| [musl libc](https://musl.libc.org/) | MIT | pwd/grp/shadow C API understanding |
| [sudo-rs](https://github.com/trifectatechfoundation/sudo-rs) | Apache-2.0 / MIT | PAM integration patterns, privilege-dropping |
| [uutils/coreutils](https://github.com/uutils/coreutils) | MIT | `uucore` infrastructure, project conventions |

**Review credits**: Code is reviewed by GitHub Copilot (automated PR review) and
Google Gemini CLI (manual security audits). Findings from both reviewers have
directly shaped the security posture of this project:

- Gemini found: TOCTOU in locking, temp file permissions, PATH injection,
  EPERM vs ESRCH in stale detection, GECOS whitespace corruption
- Copilot found: setuid-root privilege bypass (getuid vs geteuid),
  trim_start vs trim for parser correctness, PAM error handling

**Never reference**: `github.com/shadow-maint/shadow` C source (GPL-2.0+)

## Critical Rules

### GPL Clean-Room — ABSOLUTE

**NEVER read, reference, copy from, or feed into an LLM any code from `github.com/shadow-maint/shadow`** (GPL-2.0+). This includes paraphrasing or translating their logic. Violation creates a derivative work and poisons the MIT-licensed project.

Safe reference sources ONLY:
- POSIX specifications (pubs.opengroup.org)
- Man pages (man7.org) — behavioral spec, not implementation
- FreeBSD src (BSD-2-Clause)
- OpenBSD src (ISC)
- musl libc (MIT)
- sudo-rs (Apache-2.0 / MIT) — for PAM patterns and privilege-dropping

When implementing a tool: read the POSIX spec and man page for behavioral requirements, then write an original implementation. Never search for or view the C source.

### Rust Style Rules (uutils conventions)

- **No `panic!`** — never use `.unwrap()` or `panic!`. Use `unreachable!` only with a justifying comment.
- **No `std::process::exit`** — utilities must be embeddable. Return `UResult<()>` from `uumain`.
- **No `unsafe`** — enforced by `unsafe_code = "deny"` in workspace lints. Only three FFI boundary modules have `#[allow(unsafe_code)]`: `shadow-core::pam` (PAM C library), `shadow-core::crypt` (POSIX crypt(3)), and `shadow-core::process` (setuid/sigprocmask/getpwuid_r). All other code must be 100% safe Rust. This is stricter than uutils (which allows unsafe for FFI but doesn't enforce at the cargo level). When edition 2024 makes something unsafe (like `set_var`), find a safe alternative instead of adding `unsafe {}`.
- **No dead code** — enforced by `dead_code = "deny"` in workspace lints. Remove unused code, don't `#[allow(dead_code)]` it.
- **`OsStr`/`Path` over `String`** for filesystem paths — Linux paths may not be valid UTF-8. Only convert to `String`/`str` when you know the data is always valid UTF-8. Use `bstr` crate if you need string operations on `OsStr`.
- **Comments explain "why", not "what"** — if you need to describe what code does, improve the naming instead.
- **Macros sparingly** — explore simpler alternatives first.

### License

**MIT** — every file. Only these dependency licenses are acceptable: MIT, Apache-2.0, ISC, BSD-2-Clause, BSD-3-Clause, CC0-1.0, Unicode-3.0, Zlib, MPL-2.0. No GPL/LGPL dependencies ever.

## Build & Development Commands

All builds and tests run inside Docker containers. Do not build or test on the host.

```bash
# Docker workflow (primary)
docker compose run --rm debian cargo test --workspace       # Debian Trixie (glibc)
docker compose run --rm alpine cargo test --workspace       # Alpine (musl libc)
docker compose run --rm fedora cargo test --workspace       # Fedora (SELinux enforcing)
docker compose run --rm debian cargo clippy --workspace --all-targets -- -D warnings
docker compose run --rm debian cargo fmt --all --check
docker compose build --pull                                 # rebuild all images with latest base

# Cargo commands (inside container)
cargo build                          # build all crates
cargo test --workspace               # run all tests across all crates
cargo test -p shadow-core            # test only the core library
cargo test -p uu_passwd              # test a single tool crate
cargo test -p uu_passwd -- test_name # run a specific test
cargo clippy --workspace --all-targets -- -D warnings  # lint (CI enforces zero warnings)
cargo fmt --all                      # format all code
cargo fmt --all --check              # format check (CI gate)
cargo deny --all-features check all  # license + advisory + duplicate dep audit
cargo fuzz run fuzz_passwd_parse     # run a fuzz target (nightly required)
```

### Docker Test Matrix

| Target | Base Image | libc | PAM | SELinux | Why |
|--------|-----------|------|-----|---------|-----|
| `debian` | `rust:latest` (Trixie) | glibc | `libpam0g-dev` | headers only | Primary dev, reference platform |
| `alpine` | `rust:alpine` | musl | `linux-pam-dev` | none | musl libc differences, container use case |
| `fedora` | `fedora:latest` | glibc | `pam-devel` | enforcing | SELinux, different PAM stack |

## Architecture

Cargo workspace monorepo with four layers. Dependency flows strictly downward:

```
src/bin/shadow-rs.rs     (multicall binary — dispatches by argv[0])
        │
        ▼
src/uu/{tool}/           (individual tool crates)
        │
   ┌────┴────┐
   ▼         ▼
uucore    shadow-core    (shared infrastructure + domain library)
```

**`uucore`** (crates.io, v0.7) provides the common uutils infrastructure:
- `UResult<()>` / `UError` trait — error handling with exit code mapping
- `#[uucore::main]` attribute — generates `uumain` wrapper with SIGPIPE/localization
- `uucore::bin!()` macro — generates `main()` for standalone binaries
- `show_error!` / `show_warning!` macros — stderr output auto-prefixed with utility name
- `Args` trait — argument handling

**`shadow-core`** (workspace member) provides domain-specific code:
- File parsers (`/etc/passwd`, `/etc/shadow`, `/etc/group`, `/etc/gshadow`, `/etc/login.defs`)
- Atomic file writes (lock → read → write tmp → fsync → rename → unlock → invalidate nscd)
- File locking (`.lock` files with timeout and stale detection)
- PAM integration (feature-gated)
- UID/GID allocation, username validation, nscd invalidation
- SELinux context handling (feature-gated)

### Tool Crate Structure (mirrors uutils exactly)

Each tool in `src/uu/{tool}/` follows this exact layout:

```
src/uu/passwd/
├── Cargo.toml           # package = "uu_passwd", [lib] path = "src/passwd.rs"
└── src/
    ├── main.rs          # one-liner: uucore::bin!(uu_passwd);
    └── passwd.rs        # lib crate root: uumain() + uu_app()
```

**Cargo.toml pattern** (per-tool):
```toml
[package]
name = "uu_passwd"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
path = "src/passwd.rs"

[[bin]]
name = "passwd"
path = "src/main.rs"

[dependencies]
clap = { workspace = true }
uucore = { workspace = true }
shadow-core = { workspace = true, features = ["shadow", "login-defs"] }
thiserror = { workspace = true }
```

**main.rs** — identical one-liner for every tool:
```rust
uucore::bin!(uu_passwd);
```

**{tool}.rs** — every tool must export exactly two public items:
```rust
use uucore::error::UResult;

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;
    // tool implementation
    Ok(())
}

pub fn uu_app() -> Command {
    // clap Command definition
}
```

### Error Handling Pattern (uucore)

Each tool defines a private error enum implementing `uucore::error::UError`:

```rust
#[derive(Debug)]
enum PasswdError {
    PermissionDenied,
    FileBusy(String),
    // ...
}

impl UError for PasswdError {
    fn code(&self) -> i32 {
        match self {
            Self::PermissionDenied => 1,
            Self::FileBusy(_) => 5,
            // ...
        }
    }
}
```

Return `Err(PasswdError::...)` and the `#[uucore::main]` wrapper prints it via `show_error!` and returns the exit code. Do NOT `eprintln!` and then return the error — that double-prints.

For informational messages (not errors), use `uucore::show_error!` directly.

**Option constants** go in a `mod options {}` block inside the tool file:
```rust
mod options {
    pub const USER: &str = "user";
    pub const LOCK: &str = "lock";
}
```

### Testing

Four levels, all mandatory for new tools:

- **Unit tests**: in-module `#[cfg(test)] mod tests {}` blocks
- **Property tests**: `proptest` round-trip verification for all parsers (parse → serialize → parse → compare)
- **Integration tests**: `tests/by-util/test_{tool}.rs` — run `uumain()` with synthetic files, assert exit codes and file contents
- **Fuzz targets**: `fuzz/fuzz_targets/` directory, `cargo-fuzz`, all parsers — must not panic on any input

Integration tests that mutate files require root — guard with `skip_unless_root()` and run in Docker.

### Atomic File Write Pattern

Every file mutation must follow this sequence:
1. Acquire `.lock` file (with timeout + stale detection)
2. Read current file
3. Write to temp file in same directory
4. `fsync` the temp file
5. `rename` temp over original (atomic on POSIX)
6. Release `.lock` file
7. Invalidate nscd cache

## File Header

Every `.rs` file starts with:
```rust
// This file is part of the shadow-rs package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore (terms to ignore)
```

## Workspace Dependency Management

All third-party dependencies are declared centrally in the root `Cargo.toml` under `[workspace.dependencies]`. Tool crates reference them with `{ workspace = true }`. This ensures consistent versions across the workspace and makes auditing straightforward.

## Conventions

### Commits

- **Atomic commits** — small, self-contained, one logical change per commit
- **Tool-prefixed messages** — `passwd: fix buffer handling`, `shadow-core: add subid parser`, `tests/passwd: add aging test`
- Non-tool commits: `README: update`, `CI: add Fedora matrix`, `deny: update skip list`
- Do not move code around in the same commit as a behavior change

### GitHub Issues & Project Board

All work is tracked on the [shadow-rs development](https://github.com/orgs/uutils/projects) kanban board. Every bug fix or feature must start with a GitHub issue.

- Search existing issues before creating new ones
- Include: platform, version/commit, reproduction steps, expected vs actual behavior
- Reference the relevant tool in the title: `passwd: segfault on empty shadow entry`
- Label issues: `bug`, `security`, `enhancement`, `architecture`, and tool name (`passwd`, `shadow-core`)
- Add issues to the project board — they move through: **Todo → In Progress → Done**

### Pull Requests

All changes go through PRs — never push to main directly.

- One issue per PR, reference the issue: `Fixes #N`
- Title prefixed with tool name, under 70 characters: `passwd: add --status flag`
- Keep PRs small and self-contained — one logical change
- Branch naming: `fix/N-short-description` or `feat/N-short-description`
- CI must pass (clippy, fmt, tests on all 3 distros)
- Pre-commit hook runs fmt + clippy; pre-push hook runs full test suite

#### PR Review Process — MANDATORY

PRs are automatically reviewed by GitHub Copilot. **You MUST follow this process exactly:**

1. **Create the PR** — push branch, `gh pr create`
2. **Wait for Copilot review** — do NOT merge immediately. Check with `gh pr checks` and `gh api repos/uutils/shadow-rs/pulls/{N}/reviews`
3. **Read every review comment** — use `gh api repos/uutils/shadow-rs/pulls/{N}/comments` to see inline comments
4. **Address every comment** — fix the code, push new commits, and reply to each comment explaining what was changed
5. **Verify review is resolved** — all comments must be resolved before merging
6. **Only then merge** — `gh pr merge --merge --delete-branch`

Never skip or rush the review. Never merge with unresolved comments. The review exists to catch bugs in security-critical setuid-root code.

### CI Gates (local Docker, no cloud)

Pre-commit (every commit):
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`

Pre-push (every push, all 3 distros):
- `cargo test --workspace` on Debian Trixie, Alpine, Fedora
- Install hooks: `./hooks/install.sh`
