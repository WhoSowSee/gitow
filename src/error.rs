use std::{io, process::ExitStatus};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, GitowError>;

#[derive(Debug, Error)]
pub enum GitowError {
    #[error("Not a git repository")]
    NotAGitRepository,

    #[error("Git remote is not set for {0}")]
    MissingRemote(String),

    #[error("No git remotes are configured")]
    NoRemotesConfigured,

    #[error("Failed to run git {command}: {source}")]
    GitCommand {
        command: String,
        #[source]
        source: io::Error,
    },

    #[error("git {command} failed: {stderr}")]
    GitCommandFailed { command: String, stderr: String },

    #[error("File {0} is not in repository")]
    FileNotTracked(String),

    #[error("Issue feature is not supported on AWS CodeCommit")]
    AwsIssueUnsupported,

    #[error("{feature} page is not supported for {provider} remotes")]
    UnsupportedPage { feature: String, provider: String },

    #[error("Failed to run browser command {command}: {source}")]
    BrowserCommand {
        command: String,
        #[source]
        source: io::Error,
    },

    #[error("Browser command `{command}` exited with status {status}")]
    BrowserCommandFailed { command: String, status: ExitStatus },
}
