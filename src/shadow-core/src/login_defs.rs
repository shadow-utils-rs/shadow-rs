// This file is part of the shadow-rs package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore login_defs

//! Parser for `/etc/login.defs` configuration file.
//!
//! File format: `KEY VALUE` pairs, one per line. Lines starting with `#`
//! are comments. Blank lines are ignored. Keys are case-sensitive.

use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;

use crate::error::ShadowError;

/// Parsed `/etc/login.defs` configuration.
#[derive(Debug, Clone)]
pub struct LoginDefs {
    entries: HashMap<String, String>,
}

impl LoginDefs {
    /// Load and parse `/etc/login.defs` from the given path.
    ///
    /// If the file does not exist, returns an empty `LoginDefs` (this is
    /// intentional — missing `login.defs` is not an error, defaults apply).
    ///
    /// # Errors
    ///
    /// Returns `ShadowError` on I/O errors other than file-not-found.
    pub fn load(path: &Path) -> Result<Self, ShadowError> {
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self {
                    entries: HashMap::new(),
                });
            }
            Err(e) => return Err(ShadowError::IoPath(e, path.to_owned())),
        };

        let reader = std::io::BufReader::new(file);
        let mut entries = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Split on first whitespace: KEY VALUE
            if let Some((key, value)) = trimmed.split_once(|c: char| c.is_whitespace()) {
                entries.insert(key.to_string(), value.trim().to_string());
            }
        }

        Ok(Self { entries })
    }

    /// Get a string value by key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    /// Get a numeric value by key.
    #[must_use]
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.entries.get(key).and_then(|v| v.parse().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn write_login_defs(dir: &Path, content: &str) -> PathBuf {
        let path = dir.join("login.defs");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_parse_basic() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_login_defs(
            dir.path(),
            "PASS_MAX_DAYS\t99999\nPASS_MIN_DAYS\t0\nPASS_WARN_AGE\t7\n",
        );
        let defs = LoginDefs::load(&path).unwrap();
        assert_eq!(defs.get_i64("PASS_MAX_DAYS"), Some(99999));
        assert_eq!(defs.get_i64("PASS_MIN_DAYS"), Some(0));
        assert_eq!(defs.get_i64("PASS_WARN_AGE"), Some(7));
    }

    #[test]
    fn test_comments_and_blanks() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_login_defs(
            dir.path(),
            "# This is a comment\n\nPASS_MAX_DAYS 99999\n# Another comment\n",
        );
        let defs = LoginDefs::load(&path).unwrap();
        assert_eq!(defs.get_i64("PASS_MAX_DAYS"), Some(99999));
        assert_eq!(defs.get("# This is a comment"), None);
    }

    #[test]
    fn test_string_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_login_defs(
            dir.path(),
            "ENCRYPT_METHOD SHA512\nENV_PATH /bin:/usr/bin\n",
        );
        let defs = LoginDefs::load(&path).unwrap();
        assert_eq!(defs.get("ENCRYPT_METHOD"), Some("SHA512"));
        assert_eq!(defs.get("ENV_PATH"), Some("/bin:/usr/bin"));
    }

    #[test]
    fn test_missing_file_returns_empty() {
        let defs = LoginDefs::load(Path::new("/nonexistent/login.defs")).unwrap();
        assert_eq!(defs.get("PASS_MAX_DAYS"), None);
    }

    #[test]
    fn test_get_i64_invalid_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_login_defs(dir.path(), "ENCRYPT_METHOD SHA512\n");
        let defs = LoginDefs::load(&path).unwrap();
        assert_eq!(defs.get_i64("ENCRYPT_METHOD"), None);
    }
}
