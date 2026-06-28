use crate::remote::ParsedRemote;

use super::types::ProviderKind;

#[derive(Debug, Clone)]
pub(super) struct ProviderContext {
    pub repo_url: String,
    pub console_repo_url: String,
    pub azure_workitems_url: Option<String>,
}

pub(super) fn detect_provider(remote: &ParsedRemote) -> ProviderKind {
    let path_segments: Vec<&str> = remote.url_path.split('/').collect();

    if remote.domain == "github.com" {
        ProviderKind::GitHub
    } else if remote.domain == "gist.github.com" {
        ProviderKind::Gist
    } else if remote.forge.as_deref() == Some("gitea") {
        ProviderKind::Gitea
    } else if remote.domain == "bitbucket.org" {
        ProviderKind::BitbucketCloud
    } else if path_segments.len() >= 3 && path_segments[path_segments.len() - 3] == "scm" {
        ProviderKind::BitbucketServer
    } else if path_segments.len() >= 2 && path_segments[path_segments.len() - 2] == "_git" {
        ProviderKind::AzureDevOps
    } else if remote.domain.ends_with("amazonaws.com") {
        ProviderKind::AwsCodeCommit
    } else if remote.domain.contains("gitlab") {
        ProviderKind::GitLab
    } else {
        ProviderKind::Generic
    }
}

pub(super) fn provider_context(remote: &ParsedRemote, kind: ProviderKind) -> ProviderContext {
    let path_segments: Vec<&str> = remote.url_path.split('/').collect();

    match kind {
        ProviderKind::BitbucketServer => bitbucket_server_context(remote, &path_segments),
        ProviderKind::AzureDevOps => azure_devops_context(remote),
        ProviderKind::AwsCodeCommit => aws_codecommit_context(remote),
        _ => default_context(remote),
    }
}

fn bitbucket_server_context(remote: &ParsedRemote, path_segments: &[&str]) -> ProviderContext {
    let prefix = &path_segments[..path_segments.len() - 3];
    let project = path_segments[path_segments.len() - 2];
    let repository = path_segments[path_segments.len() - 1];
    let mut rebuilt_segments = prefix
        .iter()
        .map(|segment| (*segment).to_string())
        .collect::<Vec<_>>();
    rebuilt_segments.push("projects".to_string());
    rebuilt_segments.push(project.to_string());
    rebuilt_segments.push("repos".to_string());
    rebuilt_segments.push(repository.to_string());
    let repo_path = rebuilt_segments.join("/");
    let repo_url = format!("{}://{}/{}", remote.protocol, remote.domain, repo_path);

    ProviderContext {
        repo_url: repo_url.clone(),
        console_repo_url: repo_url,
        azure_workitems_url: None,
    }
}

fn azure_devops_context(remote: &ParsedRemote) -> ProviderContext {
    let repo_url = format!(
        "{}://{}/{}",
        remote.protocol, remote.domain, remote.url_path
    );
    let azure_workitems_url = if let Some((prefix, _)) = remote.url_path.split_once("/_git/") {
        Some(format!(
            "{}://{}/{}/_workitems",
            remote.protocol, remote.domain, prefix
        ))
    } else if let Some(repository) = remote.url_path.strip_prefix("_git/") {
        Some(format!(
            "{}://{}/{}/_workitems",
            remote.protocol, remote.domain, repository
        ))
    } else {
        Some(format!(
            "{}://{}/{}/_workitems",
            remote.protocol, remote.domain, remote.url_path
        ))
    };

    ProviderContext {
        repo_url: repo_url.clone(),
        console_repo_url: repo_url,
        azure_workitems_url,
    }
}

fn aws_codecommit_context(remote: &ParsedRemote) -> ProviderContext {
    let region = remote
        .domain
        .split('.')
        .nth(1)
        .unwrap_or_default()
        .to_string();
    let repository = remote.url_path.rsplit('/').next().unwrap_or_default();
    let console_repo_url = format!(
        "https://{region}.console.aws.amazon.com/codecommit/home?region={region}#/repository/{repository}"
    );
    let repo_url = format!("{console_repo_url}/browse/");

    ProviderContext {
        repo_url,
        console_repo_url,
        azure_workitems_url: None,
    }
}

fn default_context(remote: &ParsedRemote) -> ProviderContext {
    let repo_url = format!(
        "{}://{}/{}",
        remote.protocol, remote.domain, remote.url_path
    );
    ProviderContext {
        repo_url: repo_url.clone(),
        console_repo_url: repo_url,
        azure_workitems_url: None,
    }
}
