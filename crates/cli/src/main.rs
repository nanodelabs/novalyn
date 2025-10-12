#![forbid(unsafe_code)]

use mimalloc_safe::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub use novalyn as lib;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    novalyn_core::init_crypto_provider();
    match lib::cli::run() {
        Ok(exit_code) => std::process::exit(exit_code as i32),
        Err(e) => {
            if let Some(err) = e.downcast_ref::<novalyn_core::error::NovalynError>() {
                eprintln!("novalyn: {}", err);
                std::process::exit(err.exit_code());
            } else {
                eprintln!("novalyn: {}", e);
                std::process::exit(1);
            }
        }
    }
}
