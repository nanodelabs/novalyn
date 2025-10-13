//! novalyn CLI crate entry point
//!
//! This crate provides the command-line interface for novalyn, a parity-focused changelog generator.
#![forbid(unsafe_code)]

pub mod cli;
pub mod cli_def;
pub mod logging;
pub mod shells;

pub use novalyn_core::*;
