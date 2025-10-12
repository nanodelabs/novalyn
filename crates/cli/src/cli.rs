use crate::{
    github, logging,
    pipeline::{ExitCode, ReleaseOptions, run_release},
};
use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete;
use ecow::EcoVec;

pub use crate::cli_def::{Cli, Commands, Completions};

pub fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    logging::init(cli.verbose as usize);
    let cwd = cli
        .cwd
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or(std::env::current_dir()?);
    let exit = match cli.command {
        Commands::Completions(completions) => {
            let mut cmd = Cli::command();
            clap_complete::generate(
                completions.shell,
                &mut cmd,
                "novalyn",
                &mut std::io::stdout(),
            );
            ExitCode::Success
        }
        Commands::Show {
            from,
            to,
            new_version,
        } => {
            let parsed_new = new_version.and_then(|s| semver::Version::parse(&s).ok());
            let outcome = run_release(ReleaseOptions {
                cwd,
                from: from.map(|s| s.into()),
                to: to.map(|s| s.into()),
                dry_run: true,
                new_version: parsed_new,
                no_authors: true,
                exclude_authors: EcoVec::new(),
                hide_author_email: false,
                clean: false,
                sign: false,
                yes: true, // Show command doesn't need confirmation
                github_alias: false,
                github_token: None,
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
            no_github_alias,
            github_token,
        } => {
            // Read GitHub token from env if not provided
            let github_token = github_token.or_else(|| {
                std::env::var("GITHUB_TOKEN")
                    .ok()
                    .or_else(|| std::env::var("GH_TOKEN").ok())
            });

            let parsed_new = new_version.and_then(|s| semver::Version::parse(&s).ok());
            let outcome = run_release(ReleaseOptions {
                cwd: cwd.clone(),
                from: from.map(|s| s.into()),
                to: to.map(|s| s.into()),
                dry_run: !write,
                new_version: parsed_new,
                no_authors,
                exclude_authors: exclude_author.into_iter().map(|s| s.into()).collect(),
                hide_author_email,
                clean,
                sign,
                yes,
                github_alias: !no_github_alias,
                github_token: github_token.map(|s| s.into()),
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
            no_github_alias,
            github_token,
        } => {
            // Read GitHub token from env if not provided
            let github_token = github_token.or_else(|| {
                std::env::var("GITHUB_TOKEN")
                    .ok()
                    .or_else(|| std::env::var("GH_TOKEN").ok())
            });

            let parsed_new = new_version.and_then(|s| semver::Version::parse(&s).ok());
            let outcome = run_release(ReleaseOptions {
                cwd: cwd.clone(),
                from: from.map(|s| s.into()),
                to: to.map(|s| s.into()),
                dry_run,
                new_version: parsed_new,
                no_authors,
                exclude_authors: exclude_author.into_iter().map(|s| s.into()).collect(),
                hide_author_email,
                clean,
                sign,
                yes,
                github_alias: !no_github_alias,
                github_token: github_token.map(|s| s.into()),
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
                    github::sync_release(&repo, cfg.github_token.as_deref(), &tag, &body, None)
                        .await
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
