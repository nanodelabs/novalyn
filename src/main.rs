pub mod config;
pub mod repository;

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("crypto provider already installed");

    let cwd = std::env::current_dir().expect("cwd");
    // Currently config loading is synchronous; keep it as such until repo resolution async path added.
    let cfg = config::load_config(config::LoadOptions {
        cwd: &cwd,
        cli_overrides: None,
    })
    .expect("load config");
    config::log_warnings(&cfg);
    println!(
        "Loaded {} types ({} enabled)",
        cfg.types.len(),
        cfg.types.iter().filter(|t| t.enabled).count()
    );
}
