use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Initialize global tracing subscriber honoring RUST_LOG or default level.
pub fn init(verbosity: usize) {
    // Map -v occurrences to levels (error, info, debug, trace)
    let level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| format!("novalyn={level}"));
    let _ = tracing_subscriber::registry()
        .with(EnvFilter::new(filter))
        .with(fmt::layer().with_target(true))
        .try_init();
}
