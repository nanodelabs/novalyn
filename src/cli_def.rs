use clap::{ArgAction, Args, Parser, Subcommand};

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
    /// Print shell completions
    Completions(Completions),
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

#[derive(Args, Debug)]
pub struct Completions {
    /// The shell to generate completions for.
    #[arg(value_enum)]
    pub shell: crate::shells::Shell,
}
