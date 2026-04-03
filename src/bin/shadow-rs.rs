// This file is part of the shadow-rs package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

//! Multicall binary entry point for shadow-rs.
//!
//! Dispatches to the appropriate utility based on `argv[0]`.
//! When invoked as `shadow-rs <util>`, uses the first argument instead.

use std::path::Path;
use std::process::ExitCode;

/// Convert a tool's `i32` exit code to `ExitCode`.
#[allow(clippy::cast_sign_loss)] // clamp(0, 255) guarantees non-negative
fn to_exit_code(code: i32) -> ExitCode {
    ExitCode::from(code.clamp(0, 255) as u8)
}

fn main() -> ExitCode {
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();

    let binary_name = args
        .first()
        .and_then(|a| {
            Path::new(a)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
        })
        .unwrap_or_default();

    // Direct invocation via symlink (e.g., argv[0] = "passwd")
    if let Some(code) = dispatch(&binary_name, &args) {
        return to_exit_code(code);
    }

    // Multicall: `shadow-rs <util> [args...]`
    if args.len() > 1 {
        let util_name = args[1].to_string_lossy().to_string();

        if util_name == "--list" {
            print_available_utils();
            return ExitCode::SUCCESS;
        }

        if let Some(code) = dispatch(&util_name, &args[1..]) {
            return to_exit_code(code);
        }

        eprintln!("shadow-rs: unknown utility '{util_name}'");
        eprintln!("Run 'shadow-rs --list' for available utilities.");
        return ExitCode::FAILURE;
    }

    eprintln!("Usage: shadow-rs <utility> [arguments...]");
    eprintln!("Run 'shadow-rs --list' for available utilities.");
    ExitCode::FAILURE
}

fn dispatch(name: &str, args: &[std::ffi::OsString]) -> Option<i32> {
    match name {
        #[cfg(feature = "chage")]
        "chage" => Some(chage::uumain(args.iter().cloned())),
        #[cfg(feature = "chfn")]
        "chfn" => Some(chfn::uumain(args.iter().cloned())),
        #[cfg(feature = "chpasswd")]
        "chpasswd" => Some(chpasswd::uumain(args.iter().cloned())),
        #[cfg(feature = "chsh")]
        "chsh" => Some(chsh::uumain(args.iter().cloned())),
        #[cfg(feature = "groupadd")]
        "groupadd" => Some(groupadd::uumain(args.iter().cloned())),
        #[cfg(feature = "groupdel")]
        "groupdel" => Some(groupdel::uumain(args.iter().cloned())),
        #[cfg(feature = "groupmod")]
        "groupmod" => Some(groupmod::uumain(args.iter().cloned())),
        #[cfg(feature = "grpck")]
        "grpck" => Some(grpck::uumain(args.iter().cloned())),
        #[cfg(feature = "newgrp")]
        "newgrp" => Some(newgrp::uumain(args.iter().cloned())),
        #[cfg(feature = "passwd")]
        "passwd" => Some(passwd::uumain(args.iter().cloned())),
        #[cfg(feature = "pwck")]
        "pwck" => Some(pwck::uumain(args.iter().cloned())),
        #[cfg(feature = "useradd")]
        "useradd" => Some(useradd::uumain(args.iter().cloned())),
        #[cfg(feature = "userdel")]
        "userdel" => Some(userdel::uumain(args.iter().cloned())),
        #[cfg(feature = "usermod")]
        "usermod" => Some(usermod::uumain(args.iter().cloned())),
        _ => None,
    }
}

fn print_available_utils() {
    println!("Available utilities:");

    #[cfg(feature = "chage")]
    println!("  chage");
    #[cfg(feature = "chfn")]
    println!("  chfn");
    #[cfg(feature = "chpasswd")]
    println!("  chpasswd");
    #[cfg(feature = "chsh")]
    println!("  chsh");
    #[cfg(feature = "groupadd")]
    println!("  groupadd");
    #[cfg(feature = "groupdel")]
    println!("  groupdel");
    #[cfg(feature = "groupmod")]
    println!("  groupmod");
    #[cfg(feature = "grpck")]
    println!("  grpck");
    #[cfg(feature = "newgrp")]
    println!("  newgrp");
    #[cfg(feature = "passwd")]
    println!("  passwd");
    #[cfg(feature = "pwck")]
    println!("  pwck");
    #[cfg(feature = "useradd")]
    println!("  useradd");
    #[cfg(feature = "userdel")]
    println!("  userdel");
    #[cfg(feature = "usermod")]
    println!("  usermod");
}
