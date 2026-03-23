// This file is part of the shadow-rs package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore lockfile

//! File locking for `/etc/passwd`, `/etc/shadow`, etc.
//!
//! Uses `.lock` files (e.g., `/etc/passwd.lock`) with timeout and stale
//! lock detection, matching the convention used by GNU shadow-utils.
//!
//! Lock files are created atomically with `O_CREAT | O_EXCL` and contain
//! the PID of the locking process for stale detection.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use nix::fcntl::{self, OFlag};
use nix::sys::stat::Mode;
use nix::unistd;

use crate::error::ShadowError;

/// Default lock timeout (matches GNU shadow-utils `LOCK_TIMEOUT`).
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);

/// Retry interval when waiting for a lock.
const RETRY_INTERVAL: Duration = Duration::from_millis(100);

/// A held file lock. The lock is released when this value is dropped.
pub struct FileLock {
    lock_path: PathBuf,
    released: bool,
}

impl FileLock {
    /// Acquire a lock for the given file using the default timeout.
    ///
    /// Creates `{file_path}.lock` atomically. If another process holds the lock,
    /// retries until the timeout expires. Stale locks (held by dead processes)
    /// are automatically cleaned up.
    ///
    /// # Errors
    ///
    /// Returns `ShadowError::Lock` if the lock cannot be acquired within the timeout.
    pub fn acquire(file_path: &Path) -> Result<Self, ShadowError> {
        Self::acquire_with_timeout(file_path, DEFAULT_TIMEOUT)
    }

    /// Acquire a lock with a custom timeout.
    ///
    /// # Errors
    ///
    /// Returns `ShadowError::Lock` if the lock cannot be acquired within the timeout.
    pub fn acquire_with_timeout(file_path: &Path, timeout: Duration) -> Result<Self, ShadowError> {
        let lock_path = lock_path_for(file_path);
        let deadline = Instant::now() + timeout;

        loop {
            if try_create_lock(&lock_path).is_ok() {
                return Ok(Self {
                    lock_path,
                    released: false,
                });
            }

            // Lock file exists — check if it's stale.
            if is_stale_lock(&lock_path) {
                // Remove stale lock and retry immediately.
                let _ = fs::remove_file(&lock_path);
                continue;
            }

            if Instant::now() >= deadline {
                return Err(ShadowError::Lock(format!(
                    "cannot acquire lock {}: timed out after {timeout:?}",
                    lock_path.display()
                )));
            }

            thread::sleep(RETRY_INTERVAL);
        }
    }

    /// Explicitly release the lock.
    ///
    /// # Errors
    ///
    /// Returns `ShadowError::Lock` if the lock file cannot be removed.
    pub fn release(mut self) -> Result<(), ShadowError> {
        self.released = true;
        fs::remove_file(&self.lock_path).map_err(|e| {
            ShadowError::Lock(format!(
                "cannot release lock {}: {e}",
                self.lock_path.display()
            ))
        })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if !self.released {
            let _ = fs::remove_file(&self.lock_path);
        }
    }
}

/// Compute the lock file path: append `.lock` to the file path.
fn lock_path_for(file_path: &Path) -> PathBuf {
    let mut lock = file_path.as_os_str().to_owned();
    lock.push(".lock");
    PathBuf::from(lock)
}

/// Try to atomically create the lock file. Write our PID into it.
fn try_create_lock(lock_path: &Path) -> Result<(), ShadowError> {
    // O_CREAT | O_EXCL ensures atomic creation — fails if the file exists.
    let fd = fcntl::open(
        lock_path,
        OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_WRONLY,
        Mode::from_bits_truncate(0o600),
    )
    .map_err(|e| ShadowError::Lock(format!("cannot create {}: {e}", lock_path.display())))?;

    // Write our PID for stale detection.
    // SAFETY: fd is a valid file descriptor we just created.
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    let pid = unistd::getpid();
    let _ = write!(file, "{pid}");

    Ok(())
}

/// Check if an existing lock file is stale (held by a dead process).
fn is_stale_lock(lock_path: &Path) -> bool {
    let Ok(contents) = fs::read_to_string(lock_path) else {
        return false;
    };

    let Ok(pid) = contents.trim().parse::<i32>() else {
        // Cannot parse PID — treat as stale.
        return true;
    };

    if pid <= 0 {
        return true;
    }

    // Signal 0 checks if the process exists without actually sending a signal.
    let pid = nix::unistd::Pid::from_raw(pid);
    nix::sys::signal::kill(pid, None).is_err()
}

use std::os::unix::io::FromRawFd;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_path_for() {
        assert_eq!(
            lock_path_for(Path::new("/etc/shadow")),
            PathBuf::from("/etc/shadow.lock")
        );
    }

    #[test]
    fn test_acquire_and_release() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test_file");
        fs::write(&file, "data").unwrap();

        let lock = FileLock::acquire(&file).unwrap();
        assert!(lock.lock_path.exists());

        lock.release().unwrap();
        assert!(!dir.path().join("test_file.lock").exists());
    }

    #[test]
    fn test_drop_releases_lock() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test_file");
        fs::write(&file, "data").unwrap();

        {
            let _lock = FileLock::acquire(&file).unwrap();
            assert!(dir.path().join("test_file.lock").exists());
        }
        // Lock should be released by drop.
        assert!(!dir.path().join("test_file.lock").exists());
    }

    #[test]
    fn test_double_lock_times_out() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test_file");
        fs::write(&file, "data").unwrap();

        let _lock1 = FileLock::acquire(&file).unwrap();

        // Second lock should time out.
        let result = FileLock::acquire_with_timeout(&file, Duration::from_millis(200));
        assert!(result.is_err());
    }

    #[test]
    fn test_stale_lock_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test_file");
        fs::write(&file, "data").unwrap();

        // Create a lock file with a PID that doesn't exist.
        let lock_path = dir.path().join("test_file.lock");
        fs::write(&lock_path, "999999999").unwrap();

        // Should succeed because the stale lock is cleaned up.
        let lock = FileLock::acquire(&file).unwrap();
        lock.release().unwrap();
    }

    #[test]
    fn test_lock_file_contains_pid() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test_file");
        fs::write(&file, "data").unwrap();

        let lock = FileLock::acquire(&file).unwrap();
        let contents = fs::read_to_string(&lock.lock_path).unwrap();
        let pid: i32 = contents.trim().parse().unwrap();
        assert_eq!(pid, i32::try_from(std::process::id()).unwrap());

        lock.release().unwrap();
    }
}
