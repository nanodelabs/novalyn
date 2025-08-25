mod config;

fn main() {
    let cwd = std::env::current_dir().expect("cwd");
    let cfg = config::load_config(config::LoadOptions { cwd: &cwd, cli_overrides: None }).expect("load config");
    config::log_warnings(&cfg);
    println!("Loaded {} types ({} enabled)", cfg.types.len(), cfg.types.iter().filter(|t| t.enabled).count());
}
