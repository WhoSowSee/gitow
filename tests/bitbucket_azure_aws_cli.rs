mod support;

use predicates::prelude::*;

use support::{binary, create_sandbox, git};

#[test]
fn opens_bitbucket_cloud_source_view() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "set-url",
            "origin",
            "https://bitbucket.org/guyzmo/git-repo.git",
        ],
        sandbox.path(),
    );
    git(&["checkout", "-B", "bugfix/conftest_fix"], sandbox.path());

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://bitbucket.org/guyzmo/git-repo/src/bugfix/conftest_fix\n");
}

#[test]
fn opens_bitbucket_server_browse_url() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "set-url",
            "origin",
            "https://user@mybb.domain.com/root/context/scm/ppp/rrr.git",
        ],
        sandbox.path(),
    );
    git(&["checkout", "-B", "develop"], sandbox.path());

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://mybb.domain.com/root/context/projects/ppp/repos/rrr/browse?at=develop\n");
}

#[test]
fn rejects_releases_for_bitbucket_cloud() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "set-url",
            "origin",
            "https://bitbucket.org/guyzmo/git-repo.git",
        ],
        sandbox.path(),
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
    git(
        &[
            "remote",
            "set-url",
            "origin",
            "http://tfs.example.com:8080/Project/Folder/_git/Repository",
        ],
        sandbox.path(),
    );
    git(&["checkout", "-B", "mybranch"], sandbox.path());

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("http://tfs.example.com:8080/Project/Folder/_git/Repository?version=GBmybranch\n");

    git(&["checkout", "-B", "bugfix-36"], sandbox.path());
    binary()
        .args(["--print", "--issue"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("http://tfs.example.com:8080/Project/Folder/_workitems?id=36\n");
}

#[test]
fn opens_pull_requests_and_commits_for_azure_devops() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "set-url",
            "origin",
            "http://tfs.example.com:8080/Project/Folder/_git/Repository",
        ],
        sandbox.path(),
    );
    git(&["checkout", "-B", "main"], sandbox.path());

    binary()
        .args(["--print", "--pull-requests"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("http://tfs.example.com:8080/Project/Folder/_git/Repository/pullrequests\n");

    binary()
        .args(["--print", "--commits"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("http://tfs.example.com:8080/Project/Folder/_git/Repository/commits?itemVersion=GBmain\n");
}

#[test]
fn opens_aws_codecommit_repository_and_rejects_issues() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "set-url",
            "origin",
            "https://git-codecommit.us-east-1.amazonaws.com/v1/repos/repo",
        ],
        sandbox.path(),
    );
    git(&["checkout", "-B", "mybranch"], sandbox.path());

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout(
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
    git(
        &[
            "remote",
            "set-url",
            "origin",
            "https://git-codecommit.us-east-1.amazonaws.com/v1/repos/repo",
        ],
        sandbox.path(),
    );

    binary()
        .args(["--print", "--pull-requests"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout(
            "https://us-east-1.console.aws.amazon.com/codecommit/home?region=us-east-1#/repository/repo/pull-requests\n",
        );

    binary()
        .args(["--print", "--commits"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout(
            "https://us-east-1.console.aws.amazon.com/codecommit/home?region=us-east-1#/repository/repo/commits\n",
        );
}
