#![forbid(unsafe_code)]

pub mod authors;
pub mod changelog;
pub mod config;
pub mod conventional;
pub mod error;
pub mod git;
pub mod github;
pub mod parse;
pub mod pipeline;
pub mod render;
pub mod repository;
pub mod utils;

pub use ecow;
pub use semver;
pub use tokio;

/// Initialize the rustls cryptographic provider.
/// This must be called before using any TLS functionality (e.g., reqwest with wiremock).
/// It's safe to call multiple times - subsequent calls are no-ops.
pub fn init_crypto_provider() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}
