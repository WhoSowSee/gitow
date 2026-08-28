mod support;

use predicates::prelude::*;

use support::{assert_printed_url, binary, configure_remote, create_sandbox, git};

#[test]
fn opens_bitbucket_cloud_source_view() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "https://bitbucket.org/guyzmo/git-repo.git",
    );
    git(&["checkout", "-B", "bugfix/conftest_fix"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &[],
        "https://bitbucket.org/guyzmo/git-repo/src/bugfix/conftest_fix\n",
    );
}

#[test]
fn opens_bitbucket_server_browse_url() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "https://user@mybb.domain.com/root/context/scm/ppp/rrr.git",
    );
    git(&["checkout", "-B", "develop"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &[],
        "https://mybb.domain.com/root/context/projects/ppp/repos/rrr/browse?at=develop\n",
    );
}

#[test]
fn rejects_releases_for_bitbucket_cloud() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "https://bitbucket.org/guyzmo/git-repo.git",
    );

    binary()
        .args(["--print", "--releases"])
        .current_dir(sandbox.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Releases page is not supported for Bitbucket Cloud remotes",
        ));
}

#[test]
fn opens_visual_studio_branch_and_issue_urls() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "http://tfs.example.com:8080/Project/Folder/_git/Repository",
    );
    git(&["checkout", "-B", "mybranch"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &[],
        "http://tfs.example.com:8080/Project/Folder/_git/Repository?version=GBmybranch\n",
    );

    git(&["checkout", "-B", "bugfix-36"], sandbox.path());
    assert_printed_url(
        sandbox.path(),
        &["--issue"],
        "http://tfs.example.com:8080/Project/Folder/_workitems?id=36\n",
    );
}

#[test]
fn opens_pull_requests_and_commits_for_azure_devops() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "http://tfs.example.com:8080/Project/Folder/_git/Repository",
    );
    git(&["checkout", "-B", "main"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &["--pull-requests"],
        "http://tfs.example.com:8080/Project/Folder/_git/Repository/pullrequests\n",
    );
    assert_printed_url(
        sandbox.path(),
        &["--commits"],
        "http://tfs.example.com:8080/Project/Folder/_git/Repository/commits?itemVersion=GBmain\n",
    );
}

#[test]
fn opens_aws_codecommit_repository_and_rejects_issues() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "https://git-codecommit.us-east-1.amazonaws.com/v1/repos/repo",
    );
    git(&["checkout", "-B", "mybranch"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &[],
        "https://us-east-1.console.aws.amazon.com/codecommit/home?region=us-east-1#/repository/repo/browse/mybranch/--/\n",
    );

    binary()
        .args(["--print", "--issue"])
        .current_dir(sandbox.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Issue feature is not supported on AWS CodeCommit",
        ));
}

#[test]
fn opens_pull_requests_and_commits_for_codecommit() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "https://git-codecommit.us-east-1.amazonaws.com/v1/repos/repo",
    );

    assert_printed_url(
        sandbox.path(),
        &["--pull-requests"],
        "https://us-east-1.console.aws.amazon.com/codecommit/home?region=us-east-1#/repository/repo/pull-requests\n",
    );
    assert_printed_url(
        sandbox.path(),
        &["--commits"],
        "https://us-east-1.console.aws.amazon.com/codecommit/home?region=us-east-1#/repository/repo/commits\n",
    );
}
