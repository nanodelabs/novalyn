use crate::{
    github, logging,
    pipeline::{ExitCode, ReleaseOptions, run_release},
};
use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand};

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
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short = 'v', action = ArgAction::Count)]
    pub verbose: u8,
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
        /// Write output to file instead of stdout (implies --write when path is CHANGELOG.md)
        #[arg(long, value_name = "PATH")]
        output: Option<String>,
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
        /// Auto-confirm (skip prompts)
        #[arg(long)]
        yes: bool,
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
        #[arg(long)]
        yes: bool,
    },
    /// GitHub release synchronization only
    Github {
        #[arg(long)]
        tag: String,
        #[arg(long)]
        body_path: Option<String>,
    },
}

pub fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    logging::init(cli.verbose as usize);
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
                yes: true, // Show command doesn't need confirmation
            })?;
            println!("{}", outcome.version);
            ExitCode::Success
        }
        Commands::Generate {
            write,
            output,
            from,
            to,
            new_version,
            no_authors,
            exclude_author,
            hide_author_email,
            clean,
            sign,
            yes,
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
                yes,
            })?;
            if let Some(path) = output {
                std::fs::write(&path, outcome.version.to_string())?;
            }
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
            yes,
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
                yes,
            })?;
            if outcome.wrote {
                println!("Released v{}", outcome.version);
                ExitCode::Success
            } else {
                println!("No change for v{}", outcome.version);
                ExitCode::NoChange
            }
        }
        Commands::Github { tag, body_path } => {
            // Minimal body read
            let body = if let Some(path) = body_path {
                std::fs::read_to_string(path)?
            } else {
                String::new()
            };
            // attempt repo detection via config layer
            let cfg = crate::config::load_config(crate::config::LoadOptions {
                cwd: &cwd,
                cli_overrides: None,
            })?;
            if let Some(repo) = cfg.repo {
                let rt = tokio::runtime::Runtime::new()?;
                let info = rt.block_on(async move {
                    github::sync_release(&repo, cfg.github_token.as_deref(), &tag, &body).await
                });
                match info {
                    Ok(r) => {
                        println!(
                            "GitHub release {}: {} (created={}, updated={}, skipped={})",
                            r.tag, r.url, r.created, r.updated, r.skipped
                        );
                        ExitCode::Success
                    }
                    Err(e) => {
                        eprintln!("github sync error: {e}");
                        ExitCode::NoChange
                    }
                }
            } else {
                eprintln!("no repository info available");
                ExitCode::NoChange
            }
        }
    };
    Ok(exit)
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
