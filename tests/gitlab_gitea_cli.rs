mod support;

use support::{binary, create_sandbox, git};

#[test]
fn applies_gitlab_domain_and_protocol_overrides() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "set-url",
            "origin",
            "ssh://git@git.example.com:7000/XXX/YYY.git",
        ],
        sandbox.path(),
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

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("http://repo.intranet/subpath/XXX/YYY\n");
}

#[test]
fn opens_merge_requests_commits_and_releases_for_gitlab() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "set-url",
            "origin",
            "git@gitlab.example.com:user/repo.git",
        ],
        sandbox.path(),
    );
    git(&["checkout", "-B", "main"], sandbox.path());

    binary()
        .args(["--print", "--pull-requests"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://gitlab.example.com/user/repo/-/merge_requests\n");

    binary()
        .args(["--print", "--commits"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://gitlab.example.com/user/repo/-/commits/main\n");

    binary()
        .args(["--print", "--releases"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://gitlab.example.com/user/repo/-/releases\n");
}

#[test]
fn supports_gitea_branch_layout() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "set-url",
            "origin",
            "ssh://git@gitea.internal/team/repo.git",
        ],
        sandbox.path(),
    );
    git(
        &["config", "open.https://gitea.internal.forge", "gitea"],
        sandbox.path(),
    );
    git(&["checkout", "-B", "feature/awesome"], sandbox.path());

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://gitea.internal/team/repo/src/branch/feature/awesome\n");
}

#[test]
fn auto_detects_codeberg_as_forgejo_gitea_family() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "set-url",
            "origin",
            "https://codeberg.org/WhoSowSee/Soundly.git",
        ],
        sandbox.path(),
    );
    git(&["checkout", "-B", "main"], sandbox.path());

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://codeberg.org/WhoSowSee/Soundly/src/branch/main\n");
}

#[test]
fn opens_pull_requests_commits_and_releases_for_codeberg() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "set-url",
            "origin",
            "https://codeberg.org/WhoSowSee/Soundly.git",
        ],
        sandbox.path(),
    );
    git(&["checkout", "-B", "main"], sandbox.path());

    binary()
        .args(["--print", "--pull-requests"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://codeberg.org/WhoSowSee/Soundly/pulls\n");

    binary()
        .args(["--print", "--commits"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://codeberg.org/WhoSowSee/Soundly/commits/branch/main\n");

    binary()
        .args(["--print", "--releases"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://codeberg.org/WhoSowSee/Soundly/releases\n");
}
