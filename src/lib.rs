#![warn(clippy::all)]
#![allow(clippy::too_many_lines)] // Allow long functions for now
#![allow(clippy::must_use_candidate)] // Allow missing must_use for now
#![allow(clippy::module_name_repetitions)] // Allow some repetition for clarity

pub mod authors;
pub mod changelog;
pub mod cli;
pub mod config;
pub mod error;
pub mod git;
pub mod github;
pub mod logging;
pub mod parse;
pub mod pipeline;
pub mod render;
pub mod repository;
