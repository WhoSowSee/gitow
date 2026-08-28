mod support;

use support::{assert_printed_url, configure_remote, create_sandbox, git};

#[test]
fn applies_gitlab_domain_and_protocol_overrides() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "ssh://git@git.example.com:7000/XXX/YYY.git",
    );
    git(
        &[
            "config",
            "open.https://git.example.com.domain",
            "repo.intranet/subpath",
        ],
        sandbox.path(),
    );
    git(
        &["config", "open.https://git.example.com.protocol", "http"],
        sandbox.path(),
    );

    assert_printed_url(
        sandbox.path(),
        &[],
        "http://repo.intranet/subpath/XXX/YYY\n",
    );
}

#[test]
fn opens_merge_requests_commits_and_releases_for_gitlab() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "git@gitlab.example.com:user/repo.git",
    );
    git(&["checkout", "-B", "main"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &["--pull-requests"],
        "https://gitlab.example.com/user/repo/-/merge_requests\n",
    );
    assert_printed_url(
        sandbox.path(),
        &["--commits"],
        "https://gitlab.example.com/user/repo/-/commits/main\n",
    );
    assert_printed_url(
        sandbox.path(),
        &["--releases"],
        "https://gitlab.example.com/user/repo/-/releases\n",
    );
}

#[test]
fn supports_gitea_branch_layout() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "ssh://git@gitea.internal/team/repo.git",
    );
    git(
        &["config", "open.https://gitea.internal.forge", "gitea"],
        sandbox.path(),
    );
    git(&["checkout", "-B", "feature/awesome"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &[],
        "https://gitea.internal/team/repo/src/branch/feature/awesome\n",
    );
}

#[test]
fn opens_pull_requests_commits_and_releases_for_codeberg() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "https://codeberg.org/WhoSowSee/Soundly.git",
    );
    git(&["checkout", "-B", "main"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &["--pull-requests"],
        "https://codeberg.org/WhoSowSee/Soundly/pulls\n",
    );
    assert_printed_url(
        sandbox.path(),
        &["--commits"],
        "https://codeberg.org/WhoSowSee/Soundly/commits/branch/main\n",
    );
    assert_printed_url(
        sandbox.path(),
        &["--releases"],
        "https://codeberg.org/WhoSowSee/Soundly/releases\n",
    );
}
