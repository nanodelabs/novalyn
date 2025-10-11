#![forbid(unsafe_code)]

use mimalloc_safe::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub use novalyn as lib;

fn main() {
    lib::init_crypto_provider();
    match lib::cli::run() {
        Ok(exit_code) => std::process::exit(exit_code as i32),
        Err(e) => {
            // Format error to stderr with proper message
            if let Some(err) = e.downcast_ref::<lib::error::NovalynError>() {
                eprintln!("novalyn: {}", err);
                std::process::exit(err.exit_code());
            } else {
                eprintln!("novalyn: {}", e);
                std::process::exit(1);
            }
        }
    }
}
