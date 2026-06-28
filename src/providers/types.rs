#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoPage {
    PullRequests,
    Commits,
    Releases,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderUrlParts {
    pub base_url: String,
    pub branch_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProviderKind {
    GitHub,
    Gist,
    GitLab,
    Gitea,
    BitbucketCloud,
    BitbucketServer,
    AzureDevOps,
    AwsCodeCommit,
    Generic,
}

pub(super) fn provider_name(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::GitHub => "GitHub",
        ProviderKind::Gist => "GitHub Gist",
        ProviderKind::GitLab => "GitLab",
        ProviderKind::Gitea => "Gitea/Forgejo",
        ProviderKind::BitbucketCloud => "Bitbucket Cloud",
        ProviderKind::BitbucketServer => "Bitbucket Server",
        ProviderKind::AzureDevOps => "Azure DevOps",
        ProviderKind::AwsCodeCommit => "AWS CodeCommit",
        ProviderKind::Generic => "generic git hosting",
    }
}
