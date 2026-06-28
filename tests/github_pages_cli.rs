mod support;

use support::{binary, create_sandbox, git};

#[test]
fn opens_pull_requests_page_for_github() {
    let sandbox = create_sandbox();

    binary()
        .args(["--print", "--pull-requests"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/paulirish/git-open/pulls\n");
}

#[test]
fn short_alias_opens_pull_requests_page() {
    let sandbox = create_sandbox();

    binary()
        .args(["--print", "-m"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/paulirish/git-open/pulls\n");
}

#[test]
fn opens_commits_page_for_github() {
    let sandbox = create_sandbox();
    git(&["checkout", "-B", "main"], sandbox.path());

    binary()
        .args(["--print", "--commits"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/paulirish/git-open/commits/main\n");
}

#[test]
fn short_alias_opens_commits_page() {
    let sandbox = create_sandbox();
    git(&["checkout", "-B", "main"], sandbox.path());

    binary()
        .args(["--print", "-C"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/paulirish/git-open/commits/main\n");
}

#[test]
fn opens_releases_page_for_github() {
    let sandbox = create_sandbox();

    binary()
        .args(["--print", "--releases"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/paulirish/git-open/releases\n");
}

#[test]
fn short_alias_opens_releases_page() {
    let sandbox = create_sandbox();

    binary()
        .args(["--print", "-r"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/paulirish/git-open/releases\n");
}

#[test]
fn opens_all_remotes_including_origin() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "add",
            "fork",
            "git@github.com:userfork/git-open.git",
        ],
        sandbox.path(),
    );
    git(
        &[
            "remote",
            "add",
            "upstream",
            "git@github.com:upstream/git-open.git",
        ],
        sandbox.path(),
    );

    binary()
        .args(["--print", "-a"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout(
            "https://github.com/paulirish/git-open\nhttps://github.com/userfork/git-open\nhttps://github.com/upstream/git-open\n",
        );
}

#[test]
fn opens_selected_page_for_all_remotes() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "add",
            "fork",
            "git@github.com:userfork/git-open.git",
        ],
        sandbox.path(),
    );
    git(
        &[
            "remote",
            "add",
            "upstream",
            "git@github.com:upstream/git-open.git",
        ],
        sandbox.path(),
    );

    binary()
        .args(["--print", "-a", "-m"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout(
            "https://github.com/paulirish/git-open/pulls\nhttps://github.com/userfork/git-open/pulls\nhttps://github.com/upstream/git-open/pulls\n",
        );
}

#[test]
fn all_remotes_with_only_origin_opens_origin() {
    let sandbox = create_sandbox();

    binary()
        .args(["--print", "-a"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/paulirish/git-open\n");
}
