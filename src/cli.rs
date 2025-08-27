use crate::pipeline::{ExitCode, ReleaseOptions, run_release};
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "changelogen",
    version,
    about = "Generate changelogs from conventional commits."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(long)]
    pub cwd: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show the next inferred version
    Show {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long, value_name = "SEMVER")]
        new_version: Option<String>,
    },
    /// Generate release block (writes with --write)
    Generate {
        #[arg(long)]
        write: bool,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long, value_name = "SEMVER")]
        new_version: Option<String>,
        #[arg(long)]
        no_authors: bool,
        #[arg(long, value_name = "NAME_OR_EMAIL")]
        exclude_author: Vec<String>,
        #[arg(long)]
        hide_author_email: bool,
        #[arg(long)]
        clean: bool,
        #[arg(long)]
        sign: bool,
    },
    /// Run full release (bump, changelog, tag creation optional in future)
    Release {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long, value_name = "SEMVER")]
        new_version: Option<String>,
        #[arg(long)]
        no_authors: bool,
        #[arg(long, value_name = "NAME_OR_EMAIL")]
        exclude_author: Vec<String>,
        #[arg(long)]
        hide_author_email: bool,
        #[arg(long)]
        clean: bool,
        #[arg(long)]
        sign: bool,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let cwd = cli
        .cwd
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or(std::env::current_dir()?);
    let exit = match cli.command {
        Commands::Show {
            from,
            to,
            new_version,
        } => {
            let parsed_new = new_version.and_then(|s| semver::Version::parse(&s).ok());
            let outcome = run_release(ReleaseOptions {
                cwd,
                from,
                to,
                dry_run: true,
                new_version: parsed_new,
                no_authors: true,
                exclude_authors: vec![],
                hide_author_email: false,
                clean: false,
                sign: false,
            })?;
            println!("{}", outcome.version);
            ExitCode::Success
        }
        Commands::Generate {
            write,
            from,
            to,
            new_version,
            no_authors,
            exclude_author,
            hide_author_email,
            clean,
            sign,
        } => {
            let parsed_new = new_version.and_then(|s| semver::Version::parse(&s).ok());
            let outcome = run_release(ReleaseOptions {
                cwd: cwd.clone(),
                from,
                to,
                dry_run: !write,
                new_version: parsed_new,
                no_authors,
                exclude_authors: exclude_author,
                hide_author_email,
                clean,
                sign,
            })?;
            println!(
                "Generated v{} ({} commits){}",
                outcome.version,
                outcome.commit_count,
                if write {
                    if outcome.wrote {
                        " and updated CHANGELOG.md"
                    } else {
                        " (no change)"
                    }
                } else {
                    ""
                }
            );
            if !outcome.wrote && write {
                ExitCode::NoChange
            } else {
                ExitCode::Success
            }
        }
        Commands::Release {
            dry_run,
            from,
            to,
            new_version,
            no_authors,
            exclude_author,
            hide_author_email,
            clean,
            sign,
        } => {
            let parsed_new = new_version.and_then(|s| semver::Version::parse(&s).ok());
            let outcome = run_release(ReleaseOptions {
                cwd: cwd.clone(),
                from,
                to,
                dry_run,
                new_version: parsed_new,
                no_authors,
                exclude_authors: exclude_author,
                hide_author_email,
                clean,
                sign,
            })?;
            if outcome.wrote {
                println!("Released v{}", outcome.version);
                ExitCode::Success
            } else {
                println!("No change for v{}", outcome.version);
                ExitCode::NoChange
            }
        }
    };
    std::process::exit(exit as i32);
}
#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    #[test]
    fn cli_help_generates() {
        Cli::command().debug_assert();
    }
}
