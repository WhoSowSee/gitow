use crate::error::{GitowError, Result};
use crate::remote::ParsedRemote;

use super::context::{detect_provider, provider_context};
use super::types::{ProviderKind, ProviderUrlParts};
use super::utils::{digits_only, encode_path_segment, extract_first_number};

pub fn build_branch_url_parts(
    remote: &ParsedRemote,
    remote_ref: &str,
    issue: bool,
) -> Result<ProviderUrlParts> {
    let encoded_ref = encode_path_segment(remote_ref);
    let kind = detect_provider(remote);
    let context = provider_context(remote, kind);

    if issue && matches!(kind, ProviderKind::AwsCodeCommit) {
        return Err(GitowError::AwsIssueUnsupported);
    }

    let branch_ref = if issue {
        format!("/issues/{}", extract_first_number(remote_ref))
    } else {
        branch_ref_for_kind(kind, &encoded_ref)
    };

    let branch_ref = if issue && matches!(kind, ProviderKind::AzureDevOps) {
        format!("?id={}", digits_only(remote_ref))
    } else {
        branch_ref
    };

    Ok(ProviderUrlParts {
        base_url: if issue && matches!(kind, ProviderKind::AzureDevOps) {
            context.azure_workitems_url.unwrap_or(context.repo_url)
        } else {
            context.repo_url
        },
        branch_ref,
    })
}

fn branch_ref_for_kind(kind: ProviderKind, encoded_ref: &str) -> String {
    match kind {
        ProviderKind::Gitea => format!("/src/branch/{encoded_ref}"),
        ProviderKind::BitbucketCloud => format!("/src/{encoded_ref}"),
        ProviderKind::BitbucketServer => format!("/browse?at={encoded_ref}"),
        ProviderKind::AzureDevOps => format!("?version=GB{encoded_ref}"),
        ProviderKind::AwsCodeCommit => format!("{encoded_ref}/--/"),
        _ => format!("/tree/{encoded_ref}"),
    }
}

#[cfg(test)]
mod tests {
    use super::build_branch_url_parts;
    use crate::remote::ParsedRemote;

    fn remote(domain: &str, path: &str) -> ParsedRemote {
        ParsedRemote {
            domain: domain.to_string(),
            url_path: path.to_string(),
            protocol: "https".to_string(),
            forge: None,
            config_base_url: format!("https://{domain}"),
        }
    }

    #[test]
    fn builds_github_branch_urls() {
        let parts = build_branch_url_parts(&remote("github.com", "user/repo"), "feat/#42", false)
            .expect("provider output");

        assert_eq!(parts.base_url, "https://github.com/user/repo");
        assert_eq!(parts.branch_ref, "/tree/feat/%2342");
    }

    #[test]
    fn builds_bitbucket_server_urls() {
        let parts = build_branch_url_parts(
            &remote("bitbucket.example.com", "root/context/scm/team/repo"),
            "develop",
            false,
        )
        .expect("provider output");

        assert_eq!(
            parts.base_url,
            "https://bitbucket.example.com/root/context/projects/team/repos/repo"
        );
        assert_eq!(parts.branch_ref, "/browse?at=develop");
    }

    #[test]
    fn builds_azure_issue_urls() {
        let parts = build_branch_url_parts(
            &remote("gitopen.visualstudio.com", "Project/_git/Repository"),
            "bugfix-36",
            true,
        )
        .expect("provider output");

        assert_eq!(
            parts.base_url,
            "https://gitopen.visualstudio.com/Project/_workitems"
        );
        assert_eq!(parts.branch_ref, "?id=36");
    }

    #[test]
    fn builds_codecommit_urls() {
        let parts = build_branch_url_parts(
            &remote(
                "git-codecommit.us-east-1.amazonaws.com",
                "v1/repos/repository-name",
            ),
            "feature/test",
            false,
        )
        .expect("provider output");

        assert_eq!(
            parts.base_url,
            "https://us-east-1.console.aws.amazon.com/codecommit/home?region=us-east-1#/repository/repository-name/browse/"
        );
        assert_eq!(parts.branch_ref, "feature/test/--/");
    }
}
