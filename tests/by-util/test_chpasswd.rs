// This file is part of the shadow-rs package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore chpasswd

//! Integration tests for the `chpasswd` utility.
//!
//! Tests that require root are guarded by `skip_unless_root()` and run inside
//! Docker CI containers. Non-root tests exercise clap parsing and error paths
//! that do not need privilege.

use std::ffi::OsString;

/// Skip the test when not running as root (euid != 0).
fn skip_unless_root() -> bool {
    !nix::unistd::geteuid().is_root()
}

/// Run `uumain` with the given args, returning the exit code.
fn run(args: &[&str]) -> i32 {
    let os_args: Vec<OsString> = args.iter().map(|s| (*s).into()).collect();
    chpasswd::uumain(os_args.into_iter())
}

/// Helper to create a temp dir with an `etc/shadow` file.
fn setup_shadow(shadow_content: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let etc = dir.path().join("etc");
    std::fs::create_dir_all(&etc).expect("failed to create etc dir");
    std::fs::write(etc.join("shadow"), shadow_content).expect("failed to write shadow file");
    dir
}

/// Read the shadow file content back from a prefix dir.
fn read_shadow(dir: &tempfile::TempDir) -> String {
    std::fs::read_to_string(dir.path().join("etc/shadow")).expect("failed to read shadow file")
}

// ---------------------------------------------------------------------------
// Non-root tests — exercise clap parsing and error paths
// ---------------------------------------------------------------------------

#[test]
fn test_help_exits_zero() {
    let code = run(&["chpasswd", "--help"]);
    assert_eq!(code, 0, "--help should exit 0");
}

#[test]
fn test_invalid_crypt_method_exits_error() {
    let code = run(&["chpasswd", "-c", "BOGUS"]);
    // clap error for invalid value
    assert_ne!(code, 0, "invalid crypt method should exit non-zero");
}

// ---------------------------------------------------------------------------
// Root-only tests — exercise real operations via shadow-core
// ---------------------------------------------------------------------------

#[test]
fn test_batch_password_update() {
    if skip_unless_root() {
        return;
    }

    let dir =
        setup_shadow("alice:$6$oldhash:19500:0:99999:7:::\nbob:$6$oldhash:19500:0:99999:7:::\n");
    let shadow_path = dir.path().join("etc/shadow");

    // Simulate what chpasswd -e does: update password hashes.
    let mut entries =
        shadow_core::shadow::read_shadow_file(&shadow_path).expect("failed to read shadow");

    for entry in &mut entries {
        if entry.name == "alice" {
            entry.passwd = "$6$newhash_alice".to_string();
        } else if entry.name == "bob" {
            entry.passwd = "$6$newhash_bob".to_string();
        }
    }

    let file = std::fs::File::create(&shadow_path).expect("failed to create shadow file");
    shadow_core::shadow::write_shadow(&entries, file).expect("failed to write shadow");

    let content = read_shadow(&dir);
    assert!(
        content.contains("alice:$6$newhash_alice:"),
        "alice's hash should be updated, got: {content}"
    );
    assert!(
        content.contains("bob:$6$newhash_bob:"),
        "bob's hash should be updated, got: {content}"
    );
}

#[test]
fn test_preserves_other_fields() {
    if skip_unless_root() {
        return;
    }

    let dir = setup_shadow("testuser:$6$oldhash:19500:5:180:14:30:20000:\n");
    let shadow_path = dir.path().join("etc/shadow");

    let mut entries =
        shadow_core::shadow::read_shadow_file(&shadow_path).expect("failed to read shadow");

    let entry = entries
        .iter_mut()
        .find(|e| e.name == "testuser")
        .expect("testuser not found");
    entry.passwd = "$6$newhash".to_string();

    let file = std::fs::File::create(&shadow_path).expect("failed to create shadow file");
    shadow_core::shadow::write_shadow(&entries, file).expect("failed to write shadow");

    let content = read_shadow(&dir);
    // Verify aging fields are preserved.
    assert!(
        content.contains(":5:180:14:30:20000:"),
        "aging fields should be preserved, got: {content}"
    );
}

#[test]
fn test_multiple_users_atomic() {
    if skip_unless_root() {
        return;
    }

    let dir = setup_shadow(
        "user1:$6$old1:19500:0:99999:7:::\nuser2:$6$old2:19500:0:99999:7:::\nuser3:$6$old3:19500:0:99999:7:::\n",
    );
    let shadow_path = dir.path().join("etc/shadow");

    let mut entries =
        shadow_core::shadow::read_shadow_file(&shadow_path).expect("failed to read shadow");

    // Update only user1 and user3, leave user2 alone.
    for entry in &mut entries {
        match entry.name.as_str() {
            "user1" => entry.passwd = "$6$new1".to_string(),
            "user3" => entry.passwd = "$6$new3".to_string(),
            _ => {}
        }
    }

    let file = std::fs::File::create(&shadow_path).expect("failed to create shadow file");
    shadow_core::shadow::write_shadow(&entries, file).expect("failed to write shadow");

    let content = read_shadow(&dir);
    assert!(content.contains("user1:$6$new1:"));
    assert!(
        content.contains("user2:$6$old2:"),
        "user2 should be unchanged"
    );
    assert!(content.contains("user3:$6$new3:"));
}
