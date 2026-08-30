mod support;

use std::{io::Write, path::Path};

use assert_cmd::Command;
use predicates::prelude::predicate;
use tempfile::{NamedTempFile, TempDir};

use support::{assert_printed_url, binary, configure_remote, create_sandbox, git};

fn isolated_global_config() -> NamedTempFile {
    NamedTempFile::new().expect("empty global git config")
}

fn binary_with_global_config(config_path: &Path) -> Command {
    let mut command = binary();
    command.env("GIT_CONFIG_GLOBAL", config_path);
    command
}

fn assert_printed_repo_url(cwd: &Path, config: &NamedTempFile, args: &[&str], expected: &str) {
    binary_with_global_config(config.path())
        .arg("--print")
        .args(args)
        .current_dir(cwd)
        .assert()
        .success()
        .stdout(expected.as_bytes().to_vec());
}

#[test]
fn opens_complete_repository_specs_without_a_git_repository() {
    let cwd = TempDir::new().expect("working directory");

    assert_printed_url(
        cwd.path(),
        &[
            "--repo",
            "gitlab/group/project",
            "codeberg/whosowsee/mdv",
            "rust-lang/rust",
        ],
        "https://gitlab.com/group/project\nhttps://codeberg.org/whosowsee/mdv\nhttps://github.com/rust-lang/rust\n",
    );
}

#[test]
fn short_repo_flag_prefers_global_config_over_local_config_and_origin() {
    let sandbox = create_sandbox();
    git(
        &["config", "open.default.owner", "local-owner"],
        sandbox.path(),
    );
    let mut global_config = isolated_global_config();
    writeln!(global_config, "[open \"default\"]\nowner = global-owner")
        .expect("write global git config");
    global_config.flush().expect("flush global git config");

    assert_printed_repo_url(
        sandbox.path(),
        &global_config,
        &["-R", "mdv"],
        "https://github.com/global-owner/mdv\n",
    );
}

#[test]
fn repo_owner_falls_back_to_local_config() {
    let sandbox = create_sandbox();
    git(
        &["config", "open.default.owner", "local-owner"],
        sandbox.path(),
    );
    let global_config = isolated_global_config();

    assert_printed_repo_url(
        sandbox.path(),
        &global_config,
        &["--repo", "gitlab/mdv"],
        "https://gitlab.com/local-owner/mdv\n",
    );
}

#[test]
fn repo_context_falls_back_to_the_current_origin() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "git@gitlab.com:group/current.git",
    );
    let global_config = isolated_global_config();

    assert_printed_repo_url(
        sandbox.path(),
        &global_config,
        &["--repo", "mdv"],
        "https://gitlab.com/group/mdv\n",
    );
}

#[test]
fn opens_remote_only_repository_pages() {
    let cwd = TempDir::new().expect("working directory");

    for (flag, expected) in [
        (
            "--pull-requests",
            "https://github.com/WhoSowSee/mdv/pulls\n",
        ),
        ("--releases", "https://github.com/WhoSowSee/mdv/releases\n"),
    ] {
        assert_printed_url(
            cwd.path(),
            &["--repo", "github/WhoSowSee/mdv", flag],
            expected,
        );
    }
}

#[test]
fn errors_when_a_repository_owner_cannot_be_inferred() {
    let cwd = TempDir::new().expect("working directory");
    let global_config = isolated_global_config();

    binary_with_global_config(global_config.path())
        .arg("--print")
        .args(["--repo", "mdv"])
        .current_dir(cwd.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot infer a repository owner"));
}

#[test]
fn rejects_local_only_targets_with_repo_selection() {
    let cwd = TempDir::new().expect("working directory");

    binary()
        .args(["--repo", "github/WhoSowSee/mdv", "--commit"])
        .current_dir(cwd.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}
