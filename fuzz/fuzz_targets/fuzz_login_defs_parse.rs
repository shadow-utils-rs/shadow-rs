// This file is part of the shadow-rs package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

//! Fuzz target for `/etc/login.defs` parsing.
//!
//! Writes fuzzed data to a temp file, then parses it with `LoginDefs::load`.
//! Ensures the parser never panics on arbitrary input.

#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    // Only process valid UTF-8 since login.defs is a text file.
    if let Ok(s) = std::str::from_utf8(data) {
        let dir = match tempfile::tempdir() {
            Ok(d) => d,
            Err(_) => return,
        };
        let path = dir.path().join("login.defs");
        let mut file = match std::fs::File::create(&path) {
            Ok(f) => f,
            Err(_) => return,
        };
        if file.write_all(s.as_bytes()).is_err() {
            return;
        }
        drop(file);

        // Must not panic on any input — errors are fine.
        let _ = shadow_core::login_defs::LoginDefs::load(&path);
    }
});
