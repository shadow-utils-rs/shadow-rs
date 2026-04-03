# OpenBSD Security Reference for shadow-rs

Detailed analysis of OpenBSD's passwd implementation (ISC license).
Source: `cvsweb.openbsd.org/src/usr.bin/passwd/` and `src/lib/libutil/passwd.c`.

## Findings — What OpenBSD Does That We Should

### Already Implemented

| # | Pattern | Status |
|---|---------|--------|
| Signal blocking during file writes | #38 — `SignalBlocker` RAII |
| Privilege drop during PAM conversation | #39 — `PrivDrop` RAII |
| Environment sanitization | #40 — `sanitized_env()` / `harden_process()` |
| Landlock filesystem restriction | #41 — `apply_landlock()` in passwd |
| Absolute paths for subprocesses | #20 — `/usr/sbin/nscd` |
| Password zeroing | #7 — `zeroize` crate |
| Secure temp file permissions | #19 — `0o600` from creation |
| TOCTOU-resistant locking | #18 — lock-via-hard-link |
| Core dump suppression | #43 — `suppress_core_dumps()` in hardening.rs |
| Resource limit hardening | #44 — `raise_file_size_limit()` in hardening.rs |
| Zero-length output guard | #45 — in `atomic_write` |
| setuid(0) consolidation | #47 — before file operations in passwd |
| User enumeration prevention | #49 — early permission check in passwd |
| Password input interrupt handling | #48 — custom SIGINT handler removed; signal blocking covers critical sections |
| Umask reset | #51 — `UmaskGuard` RAII in lock/tmp creation |

### Not Yet Implemented

#### MEDIUM: Seccomp-BPF
Restrict syscalls to only what passwd needs after initialization.
Complex but effective — sudo-rs uses this approach.

## File References

- OpenBSD passwd.c: https://cvsweb.openbsd.org/src/usr.bin/passwd/passwd.c
- OpenBSD local_passwd.c: https://cvsweb.openbsd.org/src/usr.bin/passwd/local_passwd.c
- OpenBSD pw_init/pw_lock: https://cvsweb.openbsd.org/src/lib/libutil/passwd.c
- OpenBSD pw_dup.c: https://cvsweb.openbsd.org/src/lib/libc/gen/pw_dup.c
- OpenBSD pwd_check.c: https://cvsweb.openbsd.org/src/usr.bin/passwd/pwd_check.c
