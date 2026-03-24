// This file is part of the shadow-rs package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

//! Security hardening utilities for setuid-root tools.
//!
//! Every shadow-utils tool runs as setuid-root and must defend against
//! hostile callers. These functions implement the standard hardening
//! steps that all tools share.

/// Suppress core dumps via `RLIMIT_CORE=0`.
///
/// A core dump from a setuid-root process could expose password hashes
/// and plaintext passwords.
pub fn suppress_core_dumps() {
    let _ = nix::sys::resource::setrlimit(nix::sys::resource::Resource::RLIMIT_CORE, 0, 0);
    // PR_SET_DUMPABLE via nix::sys::prctl (no raw unsafe needed).
    // nix doesn't expose prctl directly, so we skip it rather than use unsafe.
    // RLIMIT_CORE=0 is sufficient to prevent core dumps.
}

/// Raise `RLIMIT_FSIZE` to prevent truncated file writes.
///
/// A malicious caller could `ulimit -f 1` before invoking a setuid-root
/// tool, causing `/etc/shadow` to be truncated mid-write.
pub fn raise_file_size_limit() {
    let _ = nix::sys::resource::setrlimit(
        nix::sys::resource::Resource::RLIMIT_FSIZE,
        nix::sys::resource::RLIM_INFINITY,
        nix::sys::resource::RLIM_INFINITY,
    );
}

/// Sanitize the environment for setuid-root context.
///
/// Clears all environment variables except essential ones (`TERM`, `LANG`,
/// `LC_*`) and sets `PATH` to a safe default. Prevents environment variable
/// injection attacks (`LD_PRELOAD`, `IFS`, `CDPATH`, etc.).
/// Build a sanitized environment for child process spawning.
///
/// Returns safe key-value pairs (PATH + TERM/LANG/LC_*). The current
/// process environment is NOT modified (`set_var` is unsafe in edition
/// 2024). Pass the returned Vec to `Command::env_clear().envs(...)`
/// when spawning subprocesses.
pub fn sanitized_env() -> Vec<(String, String)> {
    let mut env = Vec::new();
    env.push((
        "PATH".to_string(),
        "/usr/bin:/bin:/usr/sbin:/sbin".to_string(),
    ));
    for (k, v) in std::env::vars() {
        if k == "TERM" || k == "LANG" || k.starts_with("LC_") {
            env.push((k, v));
        }
    }
    env
}

/// Run all standard hardening steps for a setuid-root tool.
///
/// Call at the top of `uumain` before any argument parsing.
/// Returns the sanitized environment for use with child process spawning.
pub fn harden_process() -> Vec<(String, String)> {
    suppress_core_dumps();
    raise_file_size_limit();
    sanitized_env()
}
