use clap::{ArgAction, Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "novalyn",
    version,
    about = "Generate changelogs from conventional commits."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(long, short)]
    pub cwd: Option<String>,
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short = 'v', long, action = ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Print shell completions
    Completions(Completions),
    /// Show the next inferred version
    Show {
        #[arg(long, short)]
        from: Option<String>,
        #[arg(long, short)]
        to: Option<String>,
        #[arg(long, short, value_name = "SEMVER")]
        new_version: Option<String>,
    },
    /// Generate release block (writes with --write)
    Generate {
        #[arg(long, short)]
        write: bool,
        /// Write output to file instead of stdout (implies --write when path is CHANGELOG.md)
        #[arg(long, short, value_name = "PATH")]
        output: Option<String>,
        #[arg(long, short)]
        from: Option<String>,
        #[arg(long, short)]
        to: Option<String>,
        #[arg(long, value_name = "SEMVER", short)]
        new_version: Option<String>,
        #[arg(long, short = 'N')]
        no_authors: bool,
        #[arg(long, short, value_name = "NAME_OR_EMAIL")]
        exclude_author: Vec<String>,
        #[arg(long, short = 'E')]
        hide_author_email: bool,
        #[arg(long, short)]
        clean: bool,
        #[arg(long, short)]
        sign: bool,
        /// Auto-confirm (skip prompts)
        #[arg(long, short)]
        yes: bool,
        /// Disable GitHub aliasing (enabled by default, converts email addresses to @handles)
        #[arg(long, short = 'G')]
        no_github_alias: bool,
        /// GitHub token for API access (reads from GITHUB_TOKEN or GH_TOKEN env vars)
        #[arg(long, short)]
        github_token: Option<String>,
    },
    /// Run full release (bump, changelog, tag creation optional in future)
    Release {
        #[arg(long, short)]
        dry_run: bool,
        #[arg(long, short)]
        from: Option<String>,
        #[arg(long, short)]
        to: Option<String>,
        #[arg(long, value_name = "SEMVER")]
        new_version: Option<String>,
        #[arg(long, short)]
        no_authors: bool,
        #[arg(long, value_name = "NAME_OR_EMAIL")]
        exclude_author: Vec<String>,
        #[arg(long, short = 'E')]
        hide_author_email: bool,
        #[arg(long, short)]
        clean: bool,
        #[arg(long, short)]
        sign: bool,
        #[arg(long, short)]
        yes: bool,
        /// Disable GitHub aliasing (enabled by default, converts email addresses to @handles)
        #[arg(long, short = 'G')]
        no_github_alias: bool,
        /// GitHub token for API access (reads from GITHUB_TOKEN or GH_TOKEN env vars)
        #[arg(long, short)]
        github_token: Option<String>,
    },
    /// GitHub release synchronization only
    Github {
        #[arg(long, short)]
        tag: String,
        #[arg(long, short)]
        body_path: Option<String>,
    },
}

#[derive(Args, Debug)]
pub struct Completions {
    /// The shell to generate completions for.
    #[arg(value_enum)]
    pub shell: crate::shells::Shell,
}
