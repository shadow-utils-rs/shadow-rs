// This file is part of the shadow-rs package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

//! Safe wrapper around POSIX `crypt(3)` for password hashing and verification.
//!
//! This is one of only two modules (along with `pam`) where `unsafe_code`
//! is permitted, because `crypt(3)` is a C library function.

use std::ffi::CString;
use std::io::Read;

use subtle::ConstantTimeEq;

use crate::error::ShadowError;

#[link(name = "crypt")]
unsafe extern "C" {
    fn crypt(key: *const libc::c_char, salt: *const libc::c_char) -> *mut libc::c_char;
}

/// crypt(3) salt alphabet (POSIX: [a-zA-Z0-9./]).
const SALT_CHARS: &[u8] = b"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Supported crypt(3) hash methods.
#[derive(Debug, Clone, Copy)]
pub enum CryptMethod {
    /// SHA-256 ($5$)
    Sha256,
    /// SHA-512 ($6$) — recommended default
    Sha512,
    /// yescrypt ($y$)
    Yescrypt,
}

impl CryptMethod {
    /// The crypt(3) prefix for this method.
    fn prefix(self) -> &'static str {
        match self {
            Self::Sha256 => "$5$",
            Self::Sha512 => "$6$",
            Self::Yescrypt => "$y$j9T$",
        }
    }
}

/// Generate a random salt string for crypt(3).
fn generate_salt(method: CryptMethod, rounds: Option<u32>) -> Result<String, ShadowError> {
    let mut rand_bytes = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut rand_bytes))
        .map_err(|e| ShadowError::Other(format!("cannot read /dev/urandom: {e}").into()))?;

    let salt_str: String = rand_bytes
        .iter()
        .map(|&b| SALT_CHARS[(b as usize) % SALT_CHARS.len()] as char)
        .collect();

    let prefix = method.prefix();
    match rounds {
        Some(r) => Ok(format!("{prefix}rounds={r}${salt_str}$")),
        None => Ok(format!("{prefix}{salt_str}$")),
    }
}

/// Hash a plaintext password using crypt(3).
///
/// Returns the full crypt(3) hash string (e.g. `$6$salt$hash...`).
///
/// # Errors
///
/// Returns `ShadowError` if the password contains null bytes, the salt
/// cannot be generated, or crypt(3) fails.
pub fn hash_password(
    password: &str,
    method: CryptMethod,
    rounds: Option<u32>,
) -> Result<String, ShadowError> {
    let salt = generate_salt(method, rounds)?;
    let c_password = CString::new(password)
        .map_err(|_| ShadowError::Auth("password contains null byte".into()))?;
    let c_salt = CString::new(salt.as_str())
        .map_err(|_| ShadowError::Auth("salt contains null byte".into()))?;

    // SAFETY: crypt() is provided by libcrypt/glibc. Both arguments are valid
    // null-terminated C strings. The returned pointer is to a static/thread-local
    // buffer managed by crypt().
    let result = unsafe { crypt(c_password.as_ptr(), c_salt.as_ptr()) };

    if result.is_null() {
        return Err(ShadowError::Auth("crypt(3) returned NULL".into()));
    }

    // SAFETY: crypt() returned a non-null pointer to a null-terminated string.
    let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
    let hash = result_str
        .to_str()
        .map_err(|_| ShadowError::Auth("crypt(3) returned invalid UTF-8".into()))?;

    Ok(hash.to_string())
}

/// Verify a plaintext password against a crypt(3) hash.
///
/// Returns `true` if the password matches the hash, `false` otherwise.
///
/// # Errors
///
/// Returns `ShadowError` if the inputs contain null bytes.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, ShadowError> {
    let c_password = CString::new(password)
        .map_err(|_| ShadowError::Auth("password contains null byte".into()))?;
    let c_hash =
        CString::new(hash).map_err(|_| ShadowError::Auth("hash contains null byte".into()))?;

    // SAFETY: crypt() is provided by libcrypt/glibc. Both arguments are valid
    // null-terminated C strings. The returned pointer is to a static/thread-local
    // buffer managed by crypt().
    let result = unsafe { crypt(c_password.as_ptr(), c_hash.as_ptr()) };

    if result.is_null() {
        return Ok(false);
    }

    // SAFETY: crypt() returned a non-null pointer to a null-terminated string.
    let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
    let result_str = result_str.to_str().unwrap_or("");

    // Constant-time comparison prevents timing side-channel attacks
    // that could leak password hash information.
    Ok(result_str.as_bytes().ct_eq(hash.as_bytes()).into())
}
