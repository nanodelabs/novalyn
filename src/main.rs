#![forbid(unsafe_code)]

use mimalloc_safe::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub use changelogen as lib;

fn main() {
    lib::init_crypto_provider();
    match lib::cli::run() {
        Ok(exit_code) => std::process::exit(exit_code as i32),
        Err(e) => {
            // Format error to stderr with proper message
            if let Some(err) = e.downcast_ref::<lib::error::ChangelogenError>() {
                eprintln!("changelogen: {}", err);
                std::process::exit(err.exit_code());
            } else {
                eprintln!("changelogen: {}", e);
                std::process::exit(1);
            }
        }
    }
}
