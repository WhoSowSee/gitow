mod support;

use support::{assert_printed_url, configure_remote, create_sandbox, git};

#[test]
fn opens_pull_requests_page_for_github() {
    let sandbox = create_sandbox();

    assert_printed_url(
        sandbox.path(),
        &["--pull-requests"],
        "https://github.com/paulirish/git-open/pulls\n",
    );
}

#[test]
fn opens_commits_page_for_github() {
    let sandbox = create_sandbox();
    git(&["checkout", "-B", "main"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &["--commits"],
        "https://github.com/paulirish/git-open/commits/main\n",
    );
}

#[test]
fn opens_releases_page_for_github() {
    let sandbox = create_sandbox();

    assert_printed_url(
        sandbox.path(),
        &["--releases"],
        "https://github.com/paulirish/git-open/releases\n",
    );
}

#[test]
fn opens_selected_page_for_all_remotes() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "add",
        "fork",
        "git@github.com:userfork/git-open.git",
    );
    configure_remote(
        sandbox.path(),
        "add",
        "upstream",
        "git@github.com:upstream/git-open.git",
    );

    assert_printed_url(
        sandbox.path(),
        &["-a", "-m"],
        "https://github.com/paulirish/git-open/pulls\nhttps://github.com/userfork/git-open/pulls\nhttps://github.com/upstream/git-open/pulls\n",
    );
}
