use crate::error::{GitowError, Result};
use crate::remote::ParsedRemote;

use super::context::{detect_provider, provider_context};
use super::types::{ProviderKind, RepoPage, provider_name};
use super::utils::encode_path_segment;

pub fn build_commit_url(remote: &ParsedRemote, sha: &str) -> String {
    let kind = detect_provider(remote);
    let context = provider_context(remote, kind);

    match kind {
        ProviderKind::GitLab => format!("{}/-/commit/{sha}", context.repo_url),
        ProviderKind::BitbucketCloud | ProviderKind::BitbucketServer => {
            format!("{}/commits/{sha}", context.repo_url)
        }
        _ => format!("{}/commit/{sha}", context.repo_url),
    }
}

pub fn build_page_url(remote: &ParsedRemote, page: RepoPage, remote_ref: &str) -> Result<String> {
    let kind = detect_provider(remote);
    let context = provider_context(remote, kind);
    let encoded_ref = encode_path_segment(remote_ref);

    let url = match (kind, page) {
        (ProviderKind::GitHub, RepoPage::PullRequests) => format!("{}/pulls", context.repo_url),
        (ProviderKind::GitHub, RepoPage::Commits) => {
            format!("{}/commits/{encoded_ref}", context.repo_url)
        }
        (ProviderKind::GitHub, RepoPage::Releases) => format!("{}/releases", context.repo_url),

        (ProviderKind::GitLab, RepoPage::PullRequests) => {
            format!("{}/-/merge_requests", context.repo_url)
        }
        (ProviderKind::GitLab, RepoPage::Commits) => {
            format!("{}/-/commits/{encoded_ref}", context.repo_url)
        }
        (ProviderKind::GitLab, RepoPage::Releases) => {
            format!("{}/-/releases", context.repo_url)
        }

        (ProviderKind::Gitea, RepoPage::PullRequests) => format!("{}/pulls", context.repo_url),
        (ProviderKind::Gitea, RepoPage::Commits) => {
            format!("{}/commits/branch/{encoded_ref}", context.repo_url)
        }
        (ProviderKind::Gitea, RepoPage::Releases) => format!("{}/releases", context.repo_url),

        (ProviderKind::BitbucketCloud, RepoPage::PullRequests) => {
            format!("{}/pull-requests/", context.repo_url)
        }
        (ProviderKind::BitbucketCloud, RepoPage::Commits) => {
            format!("{}/commits/branch/{encoded_ref}", context.repo_url)
        }

        (ProviderKind::BitbucketServer, RepoPage::PullRequests) => {
            format!("{}/pull-requests", context.repo_url)
        }
        (ProviderKind::BitbucketServer, RepoPage::Commits) => {
            format!("{}/commits?until={encoded_ref}", context.repo_url)
        }

        (ProviderKind::AzureDevOps, RepoPage::PullRequests) => {
            format!("{}/pullrequests", context.repo_url)
        }
        (ProviderKind::AzureDevOps, RepoPage::Commits) => {
            format!("{}/commits?itemVersion=GB{encoded_ref}", context.repo_url)
        }

        (ProviderKind::AwsCodeCommit, RepoPage::PullRequests) => {
            format!("{}/pull-requests", context.console_repo_url)
        }
        (ProviderKind::AwsCodeCommit, RepoPage::Commits) => {
            format!("{}/commits", context.console_repo_url)
        }

        (ProviderKind::Gist, _) | (ProviderKind::Generic, _) | (_, RepoPage::Releases) => {
            return Err(unsupported_page(page, kind));
        }
    };

    Ok(url)
}

fn unsupported_page(page: RepoPage, provider: ProviderKind) -> GitowError {
    GitowError::UnsupportedPage {
        feature: match page {
            RepoPage::PullRequests => "Pull requests".to_string(),
            RepoPage::Commits => "Commits".to_string(),
            RepoPage::Releases => "Releases".to_string(),
        },
        provider: provider_name(provider).to_string(),
    }
}
