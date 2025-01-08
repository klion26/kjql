#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2021_compatibility)]
#![warn(missing_debug_implementations, missing_docs, unreachable_pub)]
#![doc = include_str!("../README.md")]
mod array;
pub mod errors;
mod object;
pub mod runner;
