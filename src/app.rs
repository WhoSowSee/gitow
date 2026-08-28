use std::path::Path;

use crate::{
    browser,
    cli::{Cli, OpenTarget},
    error::{GitowError, Result},
    git::{Repository, default_ssh_config_path},
    providers::{self, RepoPage},
    remote::parse_remote_url,
};

pub fn run(cli: Cli, cwd: &Path) -> Result<()> {
    let urls = resolve_urls(&cli, cwd)?;
    browser::open_urls(&urls, cli.print)
}

pub fn resolve_urls(cli: &Cli, cwd: &Path) -> Result<Vec<String>> {
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
    let tracked_remote = match branch.as_deref() {
        Some(branch) => repository.branch_remote(branch)?,
        None => None,
    };
    let default_remote = repository.default_open_remote()?;

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
    } else {
        let origin_remote = repository
            .remotes()?
            .into_iter()
            .find(|remote| remote == "origin");

        vec![
            cli.remote
                .clone()
                .or(default_remote)
                .or(origin_remote)
                .or(tracked_remote)
                .unwrap_or_else(|| "origin".to_string()),
        ]
    };

    let remote_ref = if let Some(reference) = upstream_branch.clone().or_else(|| branch.clone()) {
        reference
    } else if let Some(tag) = repository.exact_tag()? {
        tag
    } else {
        repository.head_sha()?
    };

    remote_names
        .into_iter()
        .map(|remote_name| {
            build_url_for_remote(
                &repository,
                cli,
                &remote_name,
                &remote_ref,
                branch.as_deref(),
                target,
            )
        })
        .collect()
}

fn build_url_for_remote(
    repository: &Repository,
    cli: &Cli,
    remote_name: &str,
    remote_ref: &str,
    branch: Option<&str>,
    target: OpenTarget,
) -> Result<String> {
    let git_url = repository.resolve_remote_url(remote_name)?;
    let ssh_config_path = default_ssh_config_path();
    let parsed_remote = parse_remote_url(&git_url, ssh_config_path.as_deref());
    let domain_override = repository.open_urlmatch("domain", &parsed_remote.config_base_url)?;
    let protocol_override = repository.open_urlmatch("protocol", &parsed_remote.config_base_url)?;
    let forge_override = repository.open_urlmatch("forge", &parsed_remote.config_base_url)?;
    let remote = parsed_remote.with_overrides(domain_override, protocol_override, forge_override);

    let mut open_url = match target {
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

    if let Some(suffix) = &cli.suffix {
        open_url.push('/');
        open_url.push_str(suffix);
    }

    Ok(open_url)
}
