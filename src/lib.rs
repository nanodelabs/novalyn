#![cfg_attr(not(feature = "napi"), forbid(unsafe_code))]

pub mod authors;
pub mod changelog;
pub mod cli;
pub mod cli_def;
pub mod config;
pub mod error;
pub mod git;
pub mod github;
pub mod logging;
#[cfg(feature = "napi")]
pub mod napi;
pub mod parse;
pub mod pipeline;
pub mod render;
pub mod repository;
pub mod shells;
