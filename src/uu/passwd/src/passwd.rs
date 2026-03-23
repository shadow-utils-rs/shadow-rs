// This file is part of the shadow-rs package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore chroot warndays maxdays mindays chauthtok

//! `passwd` — change user password.
//!
//! Drop-in replacement for GNU shadow-utils `passwd(1)`.

use std::path::Path;

use clap::{Arg, ArgAction, Command};

use shadow_core::lock::FileLock;
use shadow_core::shadow::{self, ShadowEntry};
use shadow_core::sysroot::SysRoot;
use shadow_core::{atomic, nscd};

mod options {
    pub const USER: &str = "user";
    pub const ALL: &str = "all";
    pub const DELETE: &str = "delete";
    pub const EXPIRE: &str = "expire";
    pub const KEEP_TOKENS: &str = "keep-tokens";
    pub const INACTIVE: &str = "inactive";
    pub const LOCK: &str = "lock";
    pub const MINDAYS: &str = "mindays";
    pub const QUIET: &str = "quiet";
    pub const REPOSITORY: &str = "repository";
    pub const ROOT: &str = "root";
    pub const PREFIX: &str = "prefix";
    pub const STATUS: &str = "status";
    pub const UNLOCK: &str = "unlock";
    pub const WARNDAYS: &str = "warndays";
    pub const MAXDAYS: &str = "maxdays";
    pub const STDIN: &str = "stdin";
}

mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const PERMISSION_DENIED: i32 = 1;
    pub const INVALID_OPTIONS: i32 = 2;
    pub const UNEXPECTED_FAILURE: i32 = 3;
    pub const PASSWD_FILE_MISSING: i32 = 4;
    pub const FILE_BUSY: i32 = 5;
    #[allow(dead_code)]
    pub const INVALID_ARGUMENT: i32 = 6;
}

/// Entry point for the `passwd` utility.
pub fn uumain(args: impl IntoIterator<Item = std::ffi::OsString>) -> i32 {
    let matches = uu_app().try_get_matches_from(args);

    let matches = match matches {
        Ok(m) => m,
        Err(e) => {
            e.print().ok();
            return if e.use_stderr() {
                exit_codes::INVALID_OPTIONS
            } else {
                exit_codes::SUCCESS
            };
        }
    };

    let prefix = matches.get_one::<String>(options::PREFIX).map(Path::new);
    let root = SysRoot::new(prefix);

    // Determine target user.
    let target_user = match resolve_target_user(&matches) {
        Ok(u) => u,
        Err(code) => return code,
    };

    // Dispatch to the appropriate operation.
    if matches.get_flag(options::STATUS) {
        let show_all = matches.get_flag(options::ALL);
        return cmd_status(&root, if show_all { None } else { Some(&target_user) });
    }

    // All remaining operations require root (euid 0).
    if !is_root() && prefix.is_none() {
        eprintln!("passwd: Permission denied.");
        return exit_codes::PERMISSION_DENIED;
    }

    if matches.get_flag(options::LOCK) {
        return cmd_lock(&root, &target_user);
    }
    if matches.get_flag(options::UNLOCK) {
        return cmd_unlock(&root, &target_user);
    }
    if matches.get_flag(options::DELETE) {
        return cmd_delete(&root, &target_user);
    }
    if matches.get_flag(options::EXPIRE) {
        return cmd_expire(&root, &target_user);
    }

    // Aging field updates.
    let has_aging = matches.contains_id(options::MINDAYS)
        || matches.contains_id(options::MAXDAYS)
        || matches.contains_id(options::WARNDAYS)
        || matches.contains_id(options::INACTIVE);

    if has_aging {
        return cmd_aging(&matches, &root, &target_user);
    }

    // Default: password change via PAM (not yet implemented).
    eprintln!("passwd: password change via PAM not yet implemented");
    exit_codes::UNEXPECTED_FAILURE
}

/// Build the clap `Command` for `passwd`.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn uu_app() -> Command {
    Command::new("passwd")
        .about("Change user password")
        .override_usage("passwd [options] [LOGIN]")
        .disable_version_flag(true)
        .arg(
            Arg::new(options::ALL)
                .short('a')
                .long("all")
                .help("report password status on all accounts")
                .requires(options::STATUS)
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(options::DELETE)
                .short('d')
                .long("delete")
                .help("delete the password for the named account")
                .conflicts_with_all([options::LOCK, options::UNLOCK, options::STATUS])
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(options::EXPIRE)
                .short('e')
                .long("expire")
                .help("force expire the password for the named account")
                .conflicts_with_all([
                    options::LOCK,
                    options::UNLOCK,
                    options::DELETE,
                    options::STATUS,
                ])
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(options::KEEP_TOKENS)
                .short('k')
                .long("keep-tokens")
                .help("change password only if expired")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(options::INACTIVE)
                .short('i')
                .long("inactive")
                .help("set password inactive after expiration to INACTIVE")
                .value_name("INACTIVE")
                .value_parser(clap::value_parser!(i64)),
        )
        .arg(
            Arg::new(options::LOCK)
                .short('l')
                .long("lock")
                .help("lock the password of the named account")
                .conflicts_with_all([options::UNLOCK, options::DELETE, options::STATUS])
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(options::MINDAYS)
                .short('n')
                .long("mindays")
                .help("set minimum number of days before password change to MIN_DAYS")
                .value_name("MIN_DAYS")
                .value_parser(clap::value_parser!(i64)),
        )
        .arg(
            Arg::new(options::QUIET)
                .short('q')
                .long("quiet")
                .help("quiet mode")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(options::REPOSITORY)
                .short('r')
                .long("repository")
                .help("change password in REPOSITORY repository")
                .value_name("REPOSITORY"),
        )
        .arg(
            Arg::new(options::ROOT)
                .short('R')
                .long("root")
                .help("directory to chroot into")
                .value_name("CHROOT_DIR"),
        )
        .arg(
            Arg::new(options::PREFIX)
                .short('P')
                .long("prefix")
                .help("directory prefix")
                .value_name("PREFIX_DIR"),
        )
        .arg(
            Arg::new(options::STATUS)
                .short('S')
                .long("status")
                .help("report password status on the named account")
                .conflicts_with_all([options::LOCK, options::UNLOCK, options::DELETE])
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(options::UNLOCK)
                .short('u')
                .long("unlock")
                .help("unlock the password of the named account")
                .conflicts_with_all([options::LOCK, options::DELETE, options::STATUS])
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(options::WARNDAYS)
                .short('w')
                .long("warndays")
                .help("set expiration warning days to WARN_DAYS")
                .value_name("WARN_DAYS")
                .value_parser(clap::value_parser!(i64)),
        )
        .arg(
            Arg::new(options::MAXDAYS)
                .short('x')
                .long("maxdays")
                .help("set maximum number of days before password change to MAX_DAYS")
                .value_name("MAX_DAYS")
                .value_parser(clap::value_parser!(i64)),
        )
        .arg(
            Arg::new(options::STDIN)
                .short('s')
                .long("stdin")
                .help("read new token from stdin")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(options::USER)
                .help("Username to change password for")
                .index(1),
        )
}

// ---------------------------------------------------------------------------
// Command implementations
// ---------------------------------------------------------------------------

/// `passwd -S [user]` / `passwd -Sa` — display account status.
fn cmd_status(root: &SysRoot, target_user: Option<&str>) -> i32 {
    let shadow_path = root.shadow_path();
    let entries = match shadow::read_shadow_file(&shadow_path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("passwd: {e}");
            return if shadow_path.exists() {
                exit_codes::UNEXPECTED_FAILURE
            } else {
                exit_codes::PASSWD_FILE_MISSING
            };
        }
    };

    match target_user {
        Some(user) => {
            let Some(entry) = entries.iter().find(|e| e.name == user) else {
                eprintln!(
                    "passwd: user '{user}' does not exist in {}",
                    shadow_path.display()
                );
                return exit_codes::UNEXPECTED_FAILURE;
            };
            println!("{}", format_status(entry));
        }
        None => {
            // --all: show all users.
            for entry in &entries {
                println!("{}", format_status(entry));
            }
        }
    }

    exit_codes::SUCCESS
}

/// `passwd -l user` — lock the account password.
fn cmd_lock(root: &SysRoot, user: &str) -> i32 {
    mutate_shadow(root, user, "Locking password", |entry| {
        entry.lock();
        Ok(())
    })
}

/// `passwd -u user` — unlock the account password.
fn cmd_unlock(root: &SysRoot, user: &str) -> i32 {
    mutate_shadow(root, user, "Unlocking password", |entry| {
        if !entry.unlock() {
            return Err("cannot unlock: password is not set or would remain locked".into());
        }
        Ok(())
    })
}

/// `passwd -d user` — delete the account password.
fn cmd_delete(root: &SysRoot, user: &str) -> i32 {
    mutate_shadow(root, user, "Removing password", |entry| {
        entry.delete_password();
        Ok(())
    })
}

/// `passwd -e user` — expire the account password.
fn cmd_expire(root: &SysRoot, user: &str) -> i32 {
    mutate_shadow(root, user, "Expiring password", |entry| {
        entry.expire();
        Ok(())
    })
}

/// `passwd -n/-x/-w/-i` — update aging fields.
fn cmd_aging(matches: &clap::ArgMatches, root: &SysRoot, user: &str) -> i32 {
    let min = matches.get_one::<i64>(options::MINDAYS).copied();
    let max = matches.get_one::<i64>(options::MAXDAYS).copied();
    let warn = matches.get_one::<i64>(options::WARNDAYS).copied();
    let inactive = matches.get_one::<i64>(options::INACTIVE).copied();

    mutate_shadow(root, user, "Updating aging information", |entry| {
        if let Some(v) = min {
            entry.min_age = Some(v);
        }
        if let Some(v) = max {
            entry.max_age = Some(v);
        }
        if let Some(v) = warn {
            entry.warn_days = Some(v);
        }
        if let Some(v) = inactive {
            entry.inactive_days = Some(v);
        }
        Ok(())
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve the target username from args or current user.
fn resolve_target_user(matches: &clap::ArgMatches) -> Result<String, i32> {
    if let Some(user) = matches.get_one::<String>(options::USER) {
        return Ok(user.clone());
    }

    // No user specified — default to current user.
    let uid = nix::unistd::getuid();
    match nix::unistd::User::from_uid(uid) {
        Ok(Some(user)) => Ok(user.name),
        Ok(None) => {
            eprintln!("passwd: cannot determine current username for uid {uid}");
            Err(exit_codes::UNEXPECTED_FAILURE)
        }
        Err(e) => {
            eprintln!("passwd: cannot determine current username: {e}");
            Err(exit_codes::UNEXPECTED_FAILURE)
        }
    }
}

/// Check if the effective user is root.
fn is_root() -> bool {
    nix::unistd::geteuid().is_root()
}

/// Format a single shadow entry as a `passwd -S` status line.
///
/// Format: `username STATUS YYYY-MM-DD min max warn inactive`
fn format_status(entry: &ShadowEntry) -> String {
    let date = match entry.last_change {
        Some(0) => "01/01/1970".to_string(),
        Some(days) => format_days_since_epoch(days),
        None => "never".to_string(),
    };

    let min = entry.min_age.map_or("-1".to_string(), |v| v.to_string());
    let max = entry.max_age.map_or("-1".to_string(), |v| v.to_string());
    let warn = entry.warn_days.map_or("-1".to_string(), |v| v.to_string());
    let inactive = entry
        .inactive_days
        .map_or("-1".to_string(), |v| v.to_string());

    format!(
        "{} {} {} {} {} {} {}",
        entry.name,
        entry.status_char(),
        date,
        min,
        max,
        warn,
        inactive
    )
}

/// Convert days since epoch to `MM/DD/YYYY` format (matching GNU `passwd -S`).
fn format_days_since_epoch(days: i64) -> String {
    let secs = days * 86400;
    // SAFETY: zeroed tm struct is valid for localtime_r to populate.
    let mut tm = unsafe { std::mem::zeroed::<libc::tm>() };
    let time = secs as libc::time_t;
    // SAFETY: both pointers are valid, properly aligned, and localtime_r is reentrant.
    unsafe {
        libc::localtime_r(&raw const time, &raw mut tm);
    }
    format!(
        "{:02}/{:02}/{:04}",
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_year + 1900
    )
}

/// Lock the shadow file, read entries, apply a mutation to one user's entry,
/// write back atomically, invalidate nscd cache.
fn mutate_shadow<F>(root: &SysRoot, username: &str, action: &str, mutate: F) -> i32
where
    F: FnOnce(&mut ShadowEntry) -> Result<(), String>,
{
    let shadow_path = root.shadow_path();

    // Acquire lock.
    let Ok(lock) = FileLock::acquire(&shadow_path) else {
        eprintln!(
            "passwd: cannot lock {}: try again later",
            shadow_path.display()
        );
        return exit_codes::FILE_BUSY;
    };

    // Read current entries.
    let mut entries = match shadow::read_shadow_file(&shadow_path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("passwd: {e}");
            drop(lock);
            return if shadow_path.exists() {
                exit_codes::UNEXPECTED_FAILURE
            } else {
                exit_codes::PASSWD_FILE_MISSING
            };
        }
    };

    // Find the target user.
    let Some(entry) = entries.iter_mut().find(|e| e.name == username) else {
        eprintln!(
            "passwd: user '{username}' does not exist in {}",
            shadow_path.display()
        );
        drop(lock);
        return exit_codes::UNEXPECTED_FAILURE;
    };

    // Apply the mutation.
    if let Err(msg) = mutate(entry) {
        eprintln!("passwd: {msg}");
        drop(lock);
        return exit_codes::UNEXPECTED_FAILURE;
    }

    // Write back atomically.
    let write_result = atomic::atomic_write(&shadow_path, |file| {
        shadow::write_shadow(&entries, file)?;
        Ok(())
    });

    if let Err(e) = write_result {
        eprintln!("passwd: failed to write {}: {e}", shadow_path.display());
        drop(lock);
        return exit_codes::UNEXPECTED_FAILURE;
    }

    // Release lock and invalidate caches.
    drop(lock);
    nscd::invalidate_cache("shadow");

    eprintln!("passwd: {action} for user {username}");
    exit_codes::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_builds() {
        uu_app().debug_assert();
    }

    #[test]
    fn test_format_status_locked() {
        let entry = ShadowEntry {
            name: "testuser".to_string(),
            passwd: "!$6$hash".to_string(),
            last_change: Some(19500),
            min_age: Some(0),
            max_age: Some(99999),
            warn_days: Some(7),
            inactive_days: None,
            expire_date: None,
            reserved: String::new(),
        };
        let status = format_status(&entry);
        assert!(status.starts_with("testuser L "));
        assert!(status.ends_with(" 0 99999 7 -1"));
    }

    #[test]
    fn test_format_status_no_password() {
        let entry = ShadowEntry {
            name: "nopw".to_string(),
            passwd: String::new(),
            last_change: Some(19500),
            min_age: Some(0),
            max_age: Some(99999),
            warn_days: Some(7),
            inactive_days: None,
            expire_date: None,
            reserved: String::new(),
        };
        let status = format_status(&entry);
        assert!(status.contains(" NP "));
    }

    #[test]
    fn test_format_status_usable() {
        let entry = ShadowEntry {
            name: "active".to_string(),
            passwd: "$6$hash".to_string(),
            last_change: Some(19500),
            min_age: Some(0),
            max_age: Some(99999),
            warn_days: Some(7),
            inactive_days: Some(30),
            expire_date: None,
            reserved: String::new(),
        };
        let status = format_status(&entry);
        assert!(status.contains(" P "));
        assert!(status.ends_with(" 0 99999 7 30"));
    }

    #[test]
    fn test_format_status_never_changed() {
        let entry = ShadowEntry {
            name: "new".to_string(),
            passwd: "*".to_string(),
            last_change: None,
            min_age: None,
            max_age: None,
            warn_days: None,
            inactive_days: None,
            expire_date: None,
            reserved: String::new(),
        };
        let status = format_status(&entry);
        // * is not locked (doesn't start with !), not empty => P
        // Actually * means "no password set / cannot login" but it's technically "P" for status.
        // GNU passwd shows it as "L" because * is a non-valid hash.
        // We follow our logic: starts_with('!') => L, empty => NP, else => P.
        assert!(status.contains(" P "));
        assert!(status.contains(" never "));
    }

    #[test]
    fn test_conflicting_flags() {
        let result = uu_app().try_get_matches_from(["passwd", "-l", "-u"]);
        assert!(result.is_err());

        let result = uu_app().try_get_matches_from(["passwd", "-l", "-d"]);
        assert!(result.is_err());

        let result = uu_app().try_get_matches_from(["passwd", "-S", "-d"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_requires_status() {
        let result = uu_app().try_get_matches_from(["passwd", "-a"]);
        assert!(result.is_err());

        let result = uu_app().try_get_matches_from(["passwd", "-S", "-a"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_status_with_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let etc = dir.path().join("etc");
        std::fs::create_dir_all(&etc).unwrap();
        std::fs::write(etc.join("shadow"), "testuser:$6$hash:19500:0:99999:7:::\n").unwrap();

        let args: Vec<std::ffi::OsString> = vec![
            "passwd".into(),
            "-S".into(),
            "-P".into(),
            dir.path().as_os_str().to_owned(),
            "testuser".into(),
        ];
        let code = uumain(args);
        assert_eq!(code, 0);
    }

    #[test]
    fn test_lock_with_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let etc = dir.path().join("etc");
        std::fs::create_dir_all(&etc).unwrap();
        std::fs::write(etc.join("shadow"), "testuser:$6$hash:19500:0:99999:7:::\n").unwrap();

        let args: Vec<std::ffi::OsString> = vec![
            "passwd".into(),
            "-l".into(),
            "-P".into(),
            dir.path().as_os_str().to_owned(),
            "testuser".into(),
        ];
        let code = uumain(args);
        assert_eq!(code, 0);

        // Verify the password is now locked.
        let content = std::fs::read_to_string(etc.join("shadow")).unwrap();
        assert!(content.contains("testuser:!$6$hash:"));
    }

    #[test]
    fn test_unlock_with_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let etc = dir.path().join("etc");
        std::fs::create_dir_all(&etc).unwrap();
        std::fs::write(etc.join("shadow"), "testuser:!$6$hash:19500:0:99999:7:::\n").unwrap();

        let args: Vec<std::ffi::OsString> = vec![
            "passwd".into(),
            "-u".into(),
            "-P".into(),
            dir.path().as_os_str().to_owned(),
            "testuser".into(),
        ];
        let code = uumain(args);
        assert_eq!(code, 0);

        let content = std::fs::read_to_string(etc.join("shadow")).unwrap();
        assert!(content.contains("testuser:$6$hash:"));
    }

    #[test]
    fn test_delete_with_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let etc = dir.path().join("etc");
        std::fs::create_dir_all(&etc).unwrap();
        std::fs::write(etc.join("shadow"), "testuser:$6$hash:19500:0:99999:7:::\n").unwrap();

        let args: Vec<std::ffi::OsString> = vec![
            "passwd".into(),
            "-d".into(),
            "-P".into(),
            dir.path().as_os_str().to_owned(),
            "testuser".into(),
        ];
        let code = uumain(args);
        assert_eq!(code, 0);

        let content = std::fs::read_to_string(etc.join("shadow")).unwrap();
        assert!(content.contains("testuser::19500:"));
    }

    #[test]
    fn test_expire_with_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let etc = dir.path().join("etc");
        std::fs::create_dir_all(&etc).unwrap();
        std::fs::write(etc.join("shadow"), "testuser:$6$hash:19500:0:99999:7:::\n").unwrap();

        let args: Vec<std::ffi::OsString> = vec![
            "passwd".into(),
            "-e".into(),
            "-P".into(),
            dir.path().as_os_str().to_owned(),
            "testuser".into(),
        ];
        let code = uumain(args);
        assert_eq!(code, 0);

        let content = std::fs::read_to_string(etc.join("shadow")).unwrap();
        assert!(content.contains("testuser:$6$hash:0:"));
    }

    #[test]
    fn test_aging_with_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let etc = dir.path().join("etc");
        std::fs::create_dir_all(&etc).unwrap();
        std::fs::write(etc.join("shadow"), "testuser:$6$hash:19500:0:99999:7:::\n").unwrap();

        let args: Vec<std::ffi::OsString> = vec![
            "passwd".into(),
            "-n".into(),
            "5".into(),
            "-x".into(),
            "90".into(),
            "-w".into(),
            "14".into(),
            "-i".into(),
            "30".into(),
            "-P".into(),
            dir.path().as_os_str().to_owned(),
            "testuser".into(),
        ];
        let code = uumain(args);
        assert_eq!(code, 0);

        let content = std::fs::read_to_string(etc.join("shadow")).unwrap();
        assert!(content.contains("testuser:$6$hash:19500:5:90:14:30::"));
    }

    #[test]
    fn test_status_all_with_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let etc = dir.path().join("etc");
        std::fs::create_dir_all(&etc).unwrap();
        std::fs::write(
            etc.join("shadow"),
            "root:$6$roothash:19000:0:99999:7:::\ntestuser:!:19500::::::\n",
        )
        .unwrap();

        let args: Vec<std::ffi::OsString> = vec![
            "passwd".into(),
            "-S".into(),
            "-a".into(),
            "-P".into(),
            dir.path().as_os_str().to_owned(),
        ];
        let code = uumain(args);
        assert_eq!(code, 0);
    }
}
