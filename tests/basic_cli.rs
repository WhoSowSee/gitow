mod support;

use predicates::prelude::*;
use tempfile::TempDir;

use support::{assert_printed_url, binary, configure_remote, create_sandbox, git};

#[test]
fn prints_help() {
    binary()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: gitow"));
}

#[test]
fn errors_outside_git_repository() {
    let temp = TempDir::new().expect("temp dir");

    binary()
        .arg("--print")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not a git repository"));
}

#[test]
fn errors_when_default_remote_is_missing() {
    let sandbox = create_sandbox();
    git(&["remote", "remove", "origin"], sandbox.path());

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Git remote is not set for origin"));
}

#[test]
fn errors_when_all_remotes_has_no_remotes() {
    let sandbox = create_sandbox();
    git(&["remote", "remove", "origin"], sandbox.path());

    binary()
        .args(["--print", "--all-remotes"])
        .current_dir(sandbox.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No git remotes are configured"));
}

#[test]
fn validates_repository_file_exists() {
    let sandbox = create_sandbox();

    binary()
        .args(["--print", "--file", "missing.txt"])
        .current_dir(sandbox.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "File missing.txt is not in repository",
        ));
}

#[test]
fn appends_suffix_and_file_path() {
    let sandbox = create_sandbox();

    assert_printed_url(
        sandbox.path(),
        &["--file", "readme.txt", "--suffix", "pulls"],
        "https://github.com/paulirish/git-open/tree/master/readme.txt/pulls\n",
    );
}

#[test]
fn preserves_http_urls_and_ports() {
    let sandbox = create_sandbox();
    configure_remote(
        sandbox.path(),
        "set-url",
        "origin",
        "http://github.com:99/user/repo.git",
    );

    assert_printed_url(sandbox.path(), &[], "http://github.com:99/user/repo\n");
}

#[test]
fn resolves_instead_of_rewrites() {
    let sandbox = create_sandbox();
    git(
        &["config", "url.http://example.com/.insteadOf", "ex:"],
        sandbox.path(),
    );
    configure_remote(sandbox.path(), "set-url", "origin", "ex:example.git");

    assert_printed_url(sandbox.path(), &[], "http://example.com/example\n");
}

#[test]
fn resolves_ssh_aliases_from_custom_ssh_config() {
    let sandbox = create_sandbox();
    let ssh_config = sandbox.path().join("ssh_config");
    std::fs::write(
        &ssh_config,
        "\
        Host basic\n\
          HostName basic.example.com\n\
        Host zero*\n\
          HostName zero.example.com\n\
        Host sub?\n\
          HostName %h.example.com\n\
        ",
    )
    .expect("write ssh config");
    configure_remote(sandbox.path(), "set-url", "origin", "basic:user/repo.git");

    binary()
        .arg("--print")
        .current_dir(sandbox.path())
        .env("GITOW_SSH_CONFIG", &ssh_config)
        .assert()
        .success()
        .stdout("https://basic.example.com/user/repo\n");
}
