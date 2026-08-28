use clap::{Arg, ArgAction, ArgGroup, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenTarget {
    Branch,
    CurrentCommit,
    Issue,
    PullRequests,
    Commits,
    Releases,
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "gitow",
    bin_name = "gitow",
    about = "Open the repository website in your browser.",
    version,
    disable_version_flag = true,
    arg(
        Arg::new("version")
            .short('v')
            .long("version")
            .action(ArgAction::Version)
            .help("Print version")
    ),
    group(
        ArgGroup::new("target")
            .args(["commit", "issue", "pull_requests", "commits_page", "releases_page"])
            .multiple(false)
    ),
    after_help = "\
Examples:
  gitow
  gitow upstream feature/my-branch
  gitow --commit
  gitow --issue
  gitow -m
  gitow -C
  gitow -r
  gitow -a
  gitow --print
"
)]
pub struct Cli {
    /// Open the current commit in the forge UI.
    #[arg(short = 'c', long = "commit")]
    pub commit: bool,

    /// Open the issue inferred from the current branch name.
    #[arg(short = 'i', long = "issue")]
    pub issue: bool,

    /// Open the pull requests / merge requests page.
    #[arg(short = 'm', long = "pull-requests")]
    pub pull_requests: bool,

    /// Open the commits page for the selected branch or ref.
    #[arg(short = 'C', long = "commits")]
    pub commits_page: bool,

    /// Open the releases page where the forge supports it.
    #[arg(short = 'r', long = "releases")]
    pub releases_page: bool,

    /// Open all configured remotes.
    #[arg(short = 'a', long = "all-remotes", conflicts_with = "remote")]
    pub all_remotes: bool,

    /// Append an arbitrary suffix to the generated URL.
    #[arg(short = 's', long = "suffix", value_name = "SUFFIX")]
    pub suffix: Option<String>,

    /// Append a repository-relative file path to the generated URL.
    #[arg(
        short = 'f',
        long = "file",
        value_name = "PATH",
        conflicts_with_all = ["commit", "issue", "pull_requests", "commits_page", "releases_page"]
    )]
    pub file: Option<String>,

    /// Print the URL instead of launching a browser.
    #[arg(short = 'p', long = "print")]
    pub print: bool,

    /// Git remote name or literal remote URL to open.
    #[arg(index = 1)]
    pub remote: Option<String>,

    /// Branch name to open. Defaults to the current branch or detached ref.
    #[arg(index = 2)]
    pub branch: Option<String>,
}

impl Cli {
    pub fn target(&self) -> OpenTarget {
        if self.commit {
            OpenTarget::CurrentCommit
        } else if self.issue {
            OpenTarget::Issue
        } else if self.pull_requests {
            OpenTarget::PullRequests
        } else if self.commits_page {
            OpenTarget::Commits
        } else if self.releases_page {
            OpenTarget::Releases
        } else {
            OpenTarget::Branch
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, OpenTarget};

    #[test]
    fn short_target_flags_select_expected_target() {
        for (flag, expected) in [
            ("-c", OpenTarget::CurrentCommit),
            ("-i", OpenTarget::Issue),
            ("-m", OpenTarget::PullRequests),
            ("-C", OpenTarget::Commits),
            ("-r", OpenTarget::Releases),
        ] {
            let cli = Cli::try_parse_from(["gitow", flag]).expect("short target flag");
            assert_eq!(cli.target(), expected);
        }
    }
}
