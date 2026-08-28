mod support;

use std::{path::Path, process::Command};

use support::{binary, create_sandbox, git};

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

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/paulirish/git-open\n");
}

#[test]
fn opens_github_branch_urls() {
    let sandbox = create_sandbox();
    git(&["checkout", "-B", "feature/mybranch"], sandbox.path());

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/paulirish/git-open/tree/feature/mybranch\n");
}

#[test]
fn prefers_origin_over_tracked_remote_when_both_exist() {
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
    git(&["config", "branch.master.remote", "fork"], sandbox.path());

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/paulirish/git-open\n");
}

#[test]
fn uses_tracked_remote_when_origin_is_missing() {
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
    git(&["config", "branch.master.remote", "fork"], sandbox.path());
    git(&["remote", "remove", "origin"], sandbox.path());

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/userfork/git-open\n");
}

#[test]
fn explicit_remote_and_branch_override_defaults() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "add",
            "upstream",
            "git@github.com:upstream/repo.git",
        ],
        sandbox.path(),
    );

    binary()
        .args(["--print", "upstream", "release/2026.04"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/upstream/repo/tree/release/2026.04\n");
}

#[test]
fn opens_upstream_branch_from_git_config() {
    let sandbox = create_sandbox();
    git(
        &[
            "remote",
            "add",
            "upstreamRemote",
            "git@github.com:user/upstream-repo.git",
        ],
        sandbox.path(),
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

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/user/upstream-repo/tree/myupstream/mybranch\n");
}

#[test]
fn opens_current_commit() {
    let sandbox = create_sandbox();
    let sha = git_output(&["rev-parse", "HEAD"], sandbox.path());

    binary()
        .args(["--print", "--commit"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout(format!(
            "https://github.com/paulirish/git-open/commit/{sha}\n"
        ));
}

#[test]
fn opens_inferred_issue_for_github() {
    let sandbox = create_sandbox();
    git(&["checkout", "-B", "issues/#12"], sandbox.path());

    binary()
        .args(["--print", "--issue"])
        .current_dir(sandbox.path())
        .assert()
        .success()
        .stdout("https://github.com/paulirish/git-open/issues/12\n");
}
