use std::path::Path;

use crate::{
    browser, cargo,
    cli::{Cli, OpenTarget},
    error::{GitowError, Result},
    git::Repository,
    providers::{self, RepoPage},
    repo_selector::resolve_repository_specs,
};

pub fn run(cli: Cli, cwd: &Path) -> Result<()> {
    let urls = resolve_urls(&cli, cwd)?;
    browser::open_urls(&urls, cli.print)
}

pub fn resolve_urls(cli: &Cli, cwd: &Path) -> Result<Vec<String>> {
    if let Some(repository_specs) = &cli.repositories {
        return build_urls_for_repositories(cli, cwd, repository_specs);
    }

    if let Some(package_names) = &cli.crates_io {
        if !package_names.is_empty() {
            return package_names
                .iter()
                .map(|package_name| {
                    cargo::crates_io_url(cwd, Some(package_name.as_str()))
                        .map(|url| with_suffix(url, cli.suffix.as_deref()))
                })
                .collect();
        }

        let url = cargo::crates_io_url(cwd, None)?;
        return Ok(vec![with_suffix(url, cli.suffix.as_deref())]);
    }

    let repository = Repository::new(cwd);
    repository.ensure_work_tree()?;
    let target = cli.target();

    let branch = match &cli.branch {
        Some(branch) => Some(branch.clone()),
        None => repository.current_branch()?,
    };

    let upstream_branch = match branch.as_deref() {
        Some(branch) => repository.branch_upstream(branch)?,
        None => None,
    };

    let remote_names = if cli.all_remotes {
        let mut remotes = repository.remotes()?;
        remotes.sort_by(|left, right| match (left == "origin", right == "origin") {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => left.cmp(right),
        });

        if remotes.is_empty() {
            return Err(GitowError::NoRemotesConfigured);
        }

        remotes
    } else if !cli.remotes.is_empty() {
        cli.remotes.clone()
    } else {
        let tracked_remote = match branch.as_deref() {
            Some(branch) => repository.branch_remote(branch)?,
            None => None,
        };
        let default_remote = repository.default_open_remote()?;
        let origin_remote = repository
            .remotes()?
            .into_iter()
            .find(|remote| remote == "origin");

        vec![
            default_remote
                .or(origin_remote)
                .or(tracked_remote)
                .unwrap_or_else(|| "origin".to_string()),
        ]
    };

    let remote_ref = if let Some(reference) = upstream_branch.or_else(|| branch.clone()) {
        reference
    } else if let Some(tag) = repository.exact_tag()? {
        tag
    } else {
        repository.head_sha()?
    };

    build_urls_for_remotes(
        &repository,
        cli,
        remote_names,
        &remote_ref,
        branch.as_deref(),
        target,
    )
}

fn build_urls_for_repositories(
    cli: &Cli,
    cwd: &Path,
    repository_specs: &[String],
) -> Result<Vec<String>> {
    let repositories = resolve_repository_specs(repository_specs, cwd)?;
    let target = cli.target();

    repositories
        .iter()
        .map(|repository| {
            let url = match target {
                OpenTarget::Branch => Ok(providers::build_repository_url(repository)),
                OpenTarget::PullRequests => {
                    providers::build_page_url(repository, RepoPage::PullRequests, "")
                }
                OpenTarget::Releases => {
                    providers::build_page_url(repository, RepoPage::Releases, "")
                }
                OpenTarget::CurrentCommit | OpenTarget::Issue | OpenTarget::Commits => {
                    unreachable!("clap rejects local-only targets with --repo")
                }
            }?;
            Ok(with_suffix(url, cli.suffix.as_deref()))
        })
        .collect()
}

fn build_urls_for_remotes(
    repository: &Repository,
    cli: &Cli,
    remote_names: Vec<String>,
    remote_ref: &str,
    branch: Option<&str>,
    target: OpenTarget,
) -> Result<Vec<String>> {
    let mut urls = Vec::with_capacity(remote_names.len());
    let mut missing_remote_errors = Vec::new();

    for remote_name in remote_names {
        match build_url_for_remote(repository, cli, &remote_name, remote_ref, branch, target) {
            Ok(url) => urls.push(url),
            Err(error @ GitowError::MissingRemote(_)) => missing_remote_errors.push(error),
            Err(error) => return Err(error),
        }
    }

    let final_error = urls.is_empty().then(|| {
        // The top-level runner prints this after the preceding errors.
        missing_remote_errors
            .pop()
            .unwrap_or(GitowError::NoRemotesConfigured)
    });

    for error in missing_remote_errors {
        eprintln!("{error}");
    }

    match final_error {
        Some(error) => Err(error),
        None => Ok(urls),
    }
}

fn build_url_for_remote(
    repository: &Repository,
    cli: &Cli,
    remote_name: &str,
    remote_ref: &str,
    branch: Option<&str>,
    target: OpenTarget,
) -> Result<String> {
    let remote = repository.configured_remote(remote_name)?;

    let open_url = match target {
        OpenTarget::Branch => {
            let parts = providers::build_branch_url_parts(&remote, remote_ref, false)?;
            let mut open_url = parts.base_url;
            let repository_file = if let Some(file) = &cli.file {
                let reference = branch.unwrap_or(remote_ref);
                let repository_file = repository
                    .repository_file(reference, file)?
                    .ok_or_else(|| GitowError::FileNotTracked(file.clone()))?;
                Some(repository_file)
            } else {
                None
            };

            if remote_ref != "master" || repository_file.is_some() {
                open_url.push_str(&parts.branch_ref);
            }

            if let Some(repository_file) = repository_file {
                open_url.push('/');
                open_url.push_str(&repository_file);
            }

            open_url
        }
        OpenTarget::CurrentCommit => {
            let sha = repository.head_sha()?;
            providers::build_commit_url(&remote, &sha)
        }
        OpenTarget::Issue => {
            let parts = providers::build_branch_url_parts(&remote, remote_ref, true)?;
            format!("{}{}", parts.base_url, parts.branch_ref)
        }
        OpenTarget::PullRequests => {
            providers::build_page_url(&remote, RepoPage::PullRequests, remote_ref)?
        }
        OpenTarget::Commits => providers::build_page_url(&remote, RepoPage::Commits, remote_ref)?,
        OpenTarget::Releases => providers::build_page_url(&remote, RepoPage::Releases, remote_ref)?,
    };

    Ok(with_suffix(open_url, cli.suffix.as_deref()))
}

fn with_suffix(mut url: String, suffix: Option<&str>) -> String {
    if let Some(suffix) = suffix {
        url.push('/');
        url.push_str(suffix);
    }
    url
}
