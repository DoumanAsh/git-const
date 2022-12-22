//!Proc macro to access git repo properties at build time.
//!
//!## Usage
//!
//!```rust
//!use git_const::{git_hash, git_short_hash};
//!
//!const SHORT_VERSION: &str = git_short_hash!();
//!const VERSION: &str = git_hash!();
//!assert_ne!(VERSION, "");
//!assert!(!VERSION.contains('\n'));
//!assert_ne!(VERSION, SHORT_VERSION);
//!assert!(VERSION.starts_with(SHORT_VERSION));
//!
//!const MASTER_VERSION: &str = git_hash!(master);
//!assert_eq!(MASTER_VERSION, VERSION); //true if current branch is master
//!```

#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

extern crate proc_macro;

use proc_macro::TokenStream;

use std::process::Command;
use core::fmt;

#[cold]
#[inline(never)]
fn compile_error(args: fmt::Arguments<'_>) -> TokenStream {
    format!("compile_error!(\"{args}\")").parse().expect("To generate compile error")
}

#[inline(always)]
fn run_git(args: &[&str]) -> Result<String, TokenStream> {
    match Command::new("git").args(args).output() {
        Ok(output) => match output.status.success() {
            true => match String::from_utf8(output.stdout) {
                Ok(output) => Ok(output),
                Err(error) => Err(compile_error(format_args!("git output is not valid utf-8: {error}"))),
            },
            false => {
                let status = output.status.code().unwrap_or(1);
                let stderr = core::str::from_utf8(&output.stderr).unwrap_or("<invalid utf-8>");
                Err(compile_error(format_args!("git failed with status {status}:\n {stderr}")))
            }
        },
        Err(error) => Err(compile_error(format_args!("git execution error: {error}"))),
    }
}

#[proc_macro]
///Retrieves git hash from current project repo
///
///Accepts branch/tag name to use as reference.
///Otherwise defaults to `HEAD`
pub fn git_hash(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let revision = match input.trim() {
        "" => "HEAD",
        input => input,
    };

    let output = match run_git(&["rev-parse", revision]) {
        Ok(output) => output,
        Err(error) => return error,
    };

    let output = output.trim();
    format!("\"{output}\"").parse().expect("generate hash string")
}

#[proc_macro]
///Retrieves short hash from current project repo
///
///Accepts branch/tag name to use as reference.
///Otherwise defaults to `HEAD`
pub fn git_short_hash(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let revision = match input.trim() {
        "" => "HEAD",
        input => input,
    };

    let output = match run_git(&["rev-parse", "--short", revision]) {
        Ok(output) => output,
        Err(error) => return error,
    };

    let output = output.trim();
    format!("\"{output}\"").parse().expect("generate hash string")
}
