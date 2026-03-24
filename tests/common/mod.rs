// This file is part of the shadow-rs package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

//! Shared test helpers for shadow-rs integration tests.
//!
//! Import with `#[path = "../common/mod.rs"] mod common;` in test files.

/// Skip the test when not running as root (euid != 0).
///
/// Returns `true` if the test should be skipped.
pub fn skip_unless_root() -> bool {
    !nix::unistd::geteuid().is_root()
}

/// Create a temp directory with synthetic `/etc/` files for testing.
///
/// Returns a `TempDir` — the directory and files are cleaned up on drop.
pub fn setup_prefix(shadow_content: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let etc = dir.path().join("etc");
    std::fs::create_dir_all(&etc).expect("failed to create etc dir");
    std::fs::write(etc.join("shadow"), shadow_content).expect("failed to write shadow");
    dir
}

/// Create a temp directory with passwd, shadow, and group files.
pub fn setup_full_prefix(
    passwd_content: &str,
    shadow_content: &str,
    group_content: &str,
) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let etc = dir.path().join("etc");
    std::fs::create_dir_all(&etc).expect("failed to create etc dir");
    std::fs::write(etc.join("passwd"), passwd_content).expect("failed to write passwd");
    std::fs::write(etc.join("shadow"), shadow_content).expect("failed to write shadow");
    std::fs::write(etc.join("group"), group_content).expect("failed to write group");
    dir
}

/// Read a file from a temp prefix directory.
pub fn read_file(dir: &tempfile::TempDir, relative: &str) -> String {
    std::fs::read_to_string(dir.path().join(relative)).expect("failed to read file")
}
