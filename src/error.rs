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

    #[error("Invalid repository spec: {0}")]
    InvalidRepositorySpec(String),

    #[error(
        "Cannot infer a repository owner; set open.default.owner or use FORGE/OWNER/REPOSITORY"
    )]
    MissingRepositoryOwner,

    #[error("Not a Cargo project")]
    NotACargoProject,

    #[error("Failed to run cargo metadata: {0}")]
    CargoMetadataCommand(#[source] io::Error),

    #[error("cargo metadata failed: {0}")]
    CargoMetadataCommandFailed(String),

    #[error("Failed to parse cargo metadata: {0}")]
    CargoMetadataParse(#[source] serde_json::Error),

    #[error("Cargo workspace does not contain any packages")]
    NoCargoPackages,

    #[error(
        "Cargo workspace contains multiple packages ({packages}); run gitow --crates-io from a package directory"
    )]
    AmbiguousCargoWorkspace { packages: String },

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
