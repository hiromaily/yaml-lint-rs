//! Private copy of the yaml-rust2 0.13.2 scanner and event parser.
//!
//! Keeping these modules inside yaml-lint-core preserves the same scanner
//! behavior when the crate is built from crates.io.

#![allow(dead_code)]

macro_rules! debug_print {
    ($($arg:tt)*) => {};
}

mod char_traits;
pub(crate) mod parser;
pub(crate) mod scanner;
