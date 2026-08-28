use std::{path::Path, process::Command};

use assert_cmd::Command as AssertCommand;
use tempfile::TempDir;

pub fn git(args: &[&str], cwd: &Path) {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .expect("run git");
    assert!(status.success(), "git {:?} failed", args);
}

pub fn create_sandbox() -> TempDir {
    let temp = TempDir::new().expect("sandbox");
    git(&["init", "-q"], temp.path());
    git(&["config", "user.email", "test@example.com"], temp.path());
    git(&["config", "user.name", "Test Runner"], temp.path());
    git(
        &[
            "remote",
            "add",
            "origin",
            "github.com:paulirish/git-open.git",
        ],
        temp.path(),
    );
    git(&["checkout", "-B", "master"], temp.path());
    std::fs::write(temp.path().join("readme.txt"), "ok\n").expect("write file");
    git(&["add", "readme.txt"], temp.path());
    git(&["commit", "-m", "add file", "-q"], temp.path());
    temp
}

pub fn binary() -> AssertCommand {
    AssertCommand::cargo_bin("gitow").expect("gitow binary")
}

pub fn assert_printed_url(cwd: &Path, args: &[&str], expected: &str) {
    binary()
        .arg("--print")
        .args(args)
        .current_dir(cwd)
        .assert()
        .success()
        .stdout(expected.as_bytes().to_vec());
}

pub fn configure_remote(cwd: &Path, action: &str, name: &str, url: &str) {
    git(&["remote", action, name, url], cwd);
}
