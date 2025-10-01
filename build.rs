use clap::CommandFactory;
use clap_complete::generate_to;
use std::env::var;
use std::fs;

include!("src/shells.rs");
include!("src/cli_def.rs");

fn main() -> std::io::Result<()> {
    let mut cmd = Cli::command();
    let completion_out_dir = Path::new(&var("OUT_DIR").unwrap()).join("completions");

    fs::create_dir_all(&completion_out_dir)?;

    for shell in Shell::value_variants() {
        generate_to(*shell, &mut cmd, "changelogen", &completion_out_dir)?;
    }

    let man_out_dir = Path::new(&var("OUT_DIR").unwrap()).join("man");
    fs::create_dir_all(&man_out_dir)?;

    // Generate man pages for main command and all subcommands
    let cmd = Cli::command();
    clap_mangen::generate_to(cmd, &man_out_dir)?;

    Ok(())
}
