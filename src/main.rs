pub use changelogen as lib;

fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("crypto provider already installed");
    if let Err(e) = lib::cli::run() {
        eprintln!("error: {e:?}");
        std::process::exit(1);
    }
}
