use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use crate::error::{GitowError, Result};

#[derive(Debug, Clone)]
pub struct Repository {
    cwd: PathBuf,
}

impl Repository {
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self { cwd: cwd.into() }
    }

    pub fn ensure_work_tree(&self) -> Result<()> {
        let output = self.run_git(["rev-parse", "--is-inside-work-tree"])?;
        if output.status.success() && trim_stdout(&output.stdout) == Some("true") {
            return Ok(());
        }

        Err(GitowError::NotAGitRepository)
    }

    pub fn current_branch(&self) -> Result<Option<String>> {
        self.optional_git(["symbolic-ref", "-q", "--short", "HEAD"])
    }

    pub fn branch_upstream(&self, branch: &str) -> Result<Option<String>> {
        let key = format!("branch.{branch}.merge");
        Ok(self
            .optional_git(["config", &key])?
            .map(|value| value.trim_start_matches("refs/heads/").to_string()))
    }

    pub fn branch_remote(&self, branch: &str) -> Result<Option<String>> {
        let key = format!("branch.{branch}.remote");
        self.optional_git(["config", &key])
    }

    pub fn default_open_remote(&self) -> Result<Option<String>> {
        self.optional_git(["config", "open.default.remote"])
    }

    pub fn resolve_remote_url(&self, remote: &str) -> Result<String> {
        let output = self.run_git(["remote", "get-url", remote])?;
        if !output.status.success() {
            if looks_like_remote_spec(remote) {
                return Ok(remote.to_string());
            }

            return Err(GitowError::MissingRemote(remote.to_string()));
        }

        let git_url = trim_stdout(&output.stdout).unwrap_or_default().to_string();
        if git_url.is_empty() {
            return Err(GitowError::MissingRemote(remote.to_string()));
        }

        Ok(git_url)
    }

    pub fn remotes(&self) -> Result<Vec<String>> {
        let output = self.run_git(["remote"])?;
        if !output.status.success() {
            return Err(self.command_failed("remote", output.stderr));
        }

        let remotes = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        Ok(remotes)
    }

    pub fn open_urlmatch(&self, key: &str, open_url: &str) -> Result<Option<String>> {
        let config_key = format!("open.{key}");
        self.optional_git(["config", "--get-urlmatch", &config_key, open_url])
    }

    pub fn exact_tag(&self) -> Result<Option<String>> {
        let output = self.run_git(["describe", "--tags", "--exact-match"])?;
        match output.status.code() {
            Some(0) => Ok(trim_stdout(&output.stdout).map(ToOwned::to_owned)),
            Some(1 | 128) => Ok(None),
            _ => Err(self.command_failed("describe --tags --exact-match", output.stderr)),
        }
    }

    pub fn head_sha(&self) -> Result<String> {
        let output = self.run_git(["rev-parse", "HEAD"])?;
        if !output.status.success() {
            return Err(self.command_failed("rev-parse HEAD", output.stderr));
        }

        Ok(trim_stdout(&output.stdout).unwrap_or_default().to_string())
    }

    pub fn repository_file(&self, reference: &str, file: &str) -> Result<Option<String>> {
        if reference.is_empty() {
            return Ok(None);
        }

        self.optional_git([
            "ls-tree",
            "--full-name",
            "--name-only",
            reference,
            "--",
            file,
        ])
    }

    fn optional_git<const N: usize>(&self, args: [&str; N]) -> Result<Option<String>> {
        let output = self.run_git(args)?;
        match output.status.code() {
            Some(0) => Ok(trim_stdout(&output.stdout).map(ToOwned::to_owned)),
            Some(1) => Ok(None),
            _ => Err(self.command_failed(&args.join(" "), output.stderr)),
        }
    }

    fn run_git<const N: usize>(&self, args: [&str; N]) -> Result<Output> {
        Command::new("git")
            .args(args)
            .current_dir(&self.cwd)
            .output()
            .map_err(|source| GitowError::GitCommand {
                command: args.join(" "),
                source,
            })
    }

    fn command_failed(&self, command: &str, stderr: Vec<u8>) -> GitowError {
        let stderr = String::from_utf8_lossy(&stderr).trim().to_string();
        GitowError::GitCommandFailed {
            command: command.to_string(),
            stderr: if stderr.is_empty() {
                "unknown git error".to_string()
            } else {
                stderr
            },
        }
    }
}

pub fn default_ssh_config_path() -> Option<PathBuf> {
    std::env::var_os("GITOW_SSH_CONFIG")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| Path::new(&home).join(".ssh").join("config"))
        })
        .filter(|path| path.exists())
}

fn trim_stdout(stdout: &[u8]) -> Option<&str> {
    let trimmed = std::str::from_utf8(stdout).ok()?.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

fn looks_like_remote_spec(remote: &str) -> bool {
    remote.contains("://") || remote.contains('@') || remote.contains(':')
}
