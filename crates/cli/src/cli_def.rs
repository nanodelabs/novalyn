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
    /// Working directory to use (defaults to current directory)
    #[arg(long, short)]
    pub cwd: Option<String>,
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short = 'v', long, action = ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate shell completions
    Completions(Completions),
    /// Show the next inferred version based on commit history and semver rules.
    Show {
        /// From tag version range
        #[arg(long, short)]
        from: Option<String>,
        /// To tag version range
        #[arg(long, short)]
        to: Option<String>,
        /// Override the inferred next version (e.g. "1.2.3")
        #[arg(long, short, value_name = "SEMVER")]
        new_version: Option<String>,
    },
    /// Generate a changelog release block for the specified commit range.
    Generate {
        /// Write the changelog to CHANGELOG.md (default: print to stdout)
        #[arg(long, short)]
        write: bool,
        /// Output path for changelog file (implies --write if path is CHANGELOG.md)
        #[arg(long, short, value_name = "PATH")]
        output: Option<String>,
        /// From tag version range
        #[arg(long, short)]
        from: Option<String>,
        /// To tag version range
        #[arg(long, short)]
        to: Option<String>,
        /// Override the inferred next version (e.g. "1.2.3")
        #[arg(long, value_name = "SEMVER", short)]
        new_version: Option<String>,
        /// Exclude contributors section from changelog
        #[arg(long, short = 'N')]
        no_authors: bool,
        /// Exclude specific authors by name or email (repeatable)
        #[arg(long, short, value_name = "NAME_OR_EMAIL")]
        exclude_author: Vec<String>,
        /// Hide authors emails
        #[arg(long, short = 'E')]
        hide_author_email: bool,
        #[arg(long, short)]
        clean: bool,
        /// Sign release
        #[arg(long, short)]
        sign: bool,
        /// Automatically confirm all prompts (non-interactive mode)
        #[arg(long, short)]
        yes: bool,
        /// Disable GitHub aliasing (enabled by default, converts email addresses to @handles)
        #[arg(long, short = 'G')]
        no_github_alias: bool,
        /// GitHub token for API access (reads from GITHUB_TOKEN or GH_TOKEN env vars)
        #[arg(long, short)]
        github_token: Option<String>,
    },
    /// Run a full release: bump version, generate changelog, create git tag, and optionally sign/tag.
    Release {
        /// Simulate the release process without making changes (preview only)
        #[arg(long, short)]
        dry_run: bool,
        /// From tag version range
        #[arg(long, short)]
        from: Option<String>,
        /// To tag version range
        #[arg(long, short)]
        to: Option<String>,
        /// Override the inferred next version (e.g. "1.2.3")
        #[arg(long, value_name = "SEMVER")]
        new_version: Option<String>,
        /// Exclude contributors section from changelog
        #[arg(long, short)]
        no_authors: bool,
        /// Exclude specific authors by name or email (repeatable)
        #[arg(long, value_name = "NAME_OR_EMAIL")]
        exclude_author: Vec<String>,
        /// Hide authors emails
        #[arg(long, short = 'E')]
        hide_author_email: bool,
        #[arg(long, short)]
        clean: bool,
        /// Sign release
        #[arg(long, short)]
        sign: bool,
        /// Automatically confirm all prompts (non-interactive mode)
        #[arg(long, short)]
        yes: bool,
        /// Disable GitHub aliasing (enabled by default, converts email addresses to @handles)
        #[arg(long, short = 'G')]
        no_github_alias: bool,
        /// GitHub token for API access (reads from GITHUB_TOKEN or GH_TOKEN env vars)
        #[arg(long, short)]
        github_token: Option<String>,
    },
    /// Synchronize GitHub releases with local changelog data.
    Github {
        /// The git tag to sync as a GitHub release
        #[arg(long, short)]
        tag: String,
        /// Path to file containing release body (defaults to changelog block)
        #[arg(long, short)]
        body_path: Option<String>,
    },
}

#[derive(Args, Debug)]
pub struct Completions {
    /// The shell to generate completions for (e.g. bash, zsh, fish, powershell).
    #[arg(value_enum)]
    pub shell: crate::shells::Shell,
}
