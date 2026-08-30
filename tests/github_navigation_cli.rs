mod support;

use std::{path::Path, process::Command};

use support::{assert_printed_url, binary, configure_remote, create_sandbox, git};

fn git_output(args: &[&str], cwd: &Path) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run git");
    assert!(output.status.success(), "git {:?} failed", args);
    String::from_utf8(output.stdout)
        .expect("utf8 git output")
        .trim()
        .to_string()
}

#[test]
fn opens_github_repository_root_for_master() {
    let sandbox = create_sandbox();

    assert_printed_url(
        sandbox.path(),
        &[],
        "https://github.com/paulirish/git-open\n",
    );
}

#[test]
fn opens_github_branch_urls() {
    let sandbox = create_sandbox();
    git(&["checkout", "-B", "feature/mybranch"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &[],
        "https://github.com/paulirish/git-open/tree/feature/mybranch\n",
    );
}

#[test]
fn origin_precedes_tracked_remote_then_falls_back_to_it() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "add",
        "fork",
        "git@github.com:userfork/git-open.git",
    );
    git(&["config", "branch.master.remote", "fork"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &[],
        "https://github.com/paulirish/git-open\n",
    );

    git(&["remote", "remove", "origin"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &[],
        "https://github.com/userfork/git-open\n",
    );
}

#[test]
fn explicit_remote_and_branch_override_defaults() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "add",
        "upstream",
        "git@github.com:upstream/repo.git",
    );

    assert_printed_url(
        sandbox.path(),
        &["upstream", "--branch", "release/2026.04"],
        "https://github.com/upstream/repo/tree/release/2026.04\n",
    );
}

#[test]
fn opens_existing_remotes_when_an_explicit_remote_is_missing() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "add",
        "gitlab",
        "git@gitlab.com:mirror/repo.git",
    );

    binary()
        .args(["--print", "origin", "missing", "gitlab"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/paulirish/git-open\nhttps://gitlab.com/mirror/repo\n")
        .stderr("Git remote is not set for missing\n");
}

#[test]
fn reports_every_missing_remote_when_none_exist() {
    let sandbox = create_sandbox();

    binary()
        .args(["--print", "gitlab", "codeberg"])
        .current_dir(sandbox.path())
        .assert()
        .failure()
        .stdout("")
        .stderr("Git remote is not set for gitlab\nGit remote is not set for codeberg\n");
}

#[test]
fn opens_upstream_branch_from_git_config() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "add",
        "upstreamRemote",
        "git@github.com:user/upstream-repo.git",
    );
    git(&["checkout", "-B", "mybranch"], sandbox.path());
    git(
        &[
            "config",
            "branch.mybranch.merge",
            "refs/heads/myupstream/mybranch",
        ],
        sandbox.path(),
    );
    git(
        &["config", "branch.mybranch.remote", "upstreamRemote"],
        sandbox.path(),
    );
    git(&["remote", "remove", "origin"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &[],
        "https://github.com/user/upstream-repo/tree/myupstream/mybranch\n",
    );
}

#[test]
fn opens_current_commit() {
    let sandbox = create_sandbox();
    let sha = git_output(&["rev-parse", "HEAD"], sandbox.path());

    let expected = format!("https://github.com/paulirish/git-open/commit/{sha}\n");
    assert_printed_url(sandbox.path(), &["--commit"], &expected);
}

#[test]
fn opens_inferred_issue_for_github() {
    let sandbox = create_sandbox();
    git(&["checkout", "-B", "issues/#12"], sandbox.path());

    assert_printed_url(
        sandbox.path(),
        &["--issue"],
        "https://github.com/paulirish/git-open/issues/12\n",
    );
}
