use clap::Parser;

/// Check if local Git repositories are safe to delete
#[derive(Parser, Debug)]
#[command(name = "repo-check")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Include current directory (./) as a target
    #[arg(long)]
    pub include_dot: bool,

    /// Show only SAFE repositories
    #[arg(long, conflicts_with_all = ["only_unsafe", "only_unknown"])]
    pub only_safe: bool,

    /// Show only UNSAFE repositories
    #[arg(long, conflicts_with_all = ["only_safe", "only_unknown"])]
    pub only_unsafe: bool,

    /// Show only UNKNOWN repositories
    #[arg(long, conflicts_with_all = ["only_safe", "only_unsafe"])]
    pub only_unknown: bool,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Ignore untracked files when checking for uncommitted changes
    #[arg(long)]
    pub ignore_untracked: bool,

    /// Include UNKNOWN repositories in delete candidates
    #[arg(long)]
    pub allow_unknown: bool,

    /// Delete SAFE repositories (requires --yes for non-interactive mode)
    #[arg(long)]
    pub delete: bool,

    /// Skip confirmation prompts (for CI/scripts)
    #[arg(long, requires = "delete")]
    pub yes: bool,

    /// Move to trash instead of permanent deletion (falls back to rm -rf if unavailable)
    #[arg(long)]
    pub trash: bool,

    /// Target directory to scan (defaults to current directory)
    #[arg(default_value = ".")]
    pub path: String,
}
