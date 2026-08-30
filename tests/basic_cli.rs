mod support;

use std::{fs, path::Path};

use predicates::prelude::*;
use tempfile::TempDir;

use support::{assert_printed_url, binary, configure_remote, create_sandbox, git};

#[test]
fn prints_help() {
    binary().arg("-h").assert().success().stdout(
        predicate::str::contains("Usage: gitow").and(predicate::str::contains("-v, --version")),
    );
}

#[test]
fn prints_version_without_a_repository() {
    let temp = TempDir::new().expect("temp dir");

    for flag in ["--version", "-v"] {
        binary()
            .arg(flag)
            .current_dir(temp.path())
            .assert()
            .success()
            .stdout(format!("gitow {}\n", env!("CARGO_PKG_VERSION")))
            .stderr("");
    }
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
fn opens_crates_io_page_for_cargo_package_without_git_repository() {
    let cargo_project = TempDir::new().expect("cargo project");
    create_cargo_package(cargo_project.path(), "manifest-package");

    assert_crates_io_url(
        &cargo_project.path().join("src"),
        &["--crates-io"],
        "manifest-package",
    );
}

#[test]
fn handles_named_and_implicit_crates_outside_cargo_projects() {
    let temp = TempDir::new().expect("temp dir");

    assert_printed_url(
        temp.path(),
        &["-x", "k580-core", "kr580"],
        "https://crates.io/crates/k580-core\nhttps://crates.io/crates/kr580\n",
    );

    binary()
        .args(["--print", "--crates-io"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr("Not a Cargo project\n");
}

#[test]
fn resolves_multi_package_workspace_from_member_or_root() {
    let workspace = TempDir::new().expect("cargo workspace");
    create_cargo_workspace(workspace.path(), &["beta", "alpha"]);

    assert_crates_io_url(&workspace.path().join("beta"), &["--crates-io"], "beta");

    binary()
        .args(["--print", "--crates-io"])
        .current_dir(workspace.path())
        .assert()
        .failure()
        .stderr(
            "Cargo workspace contains multiple packages (alpha, beta); run gitow --crates-io from a package directory\n",
        );
}

#[test]
fn opens_only_package_from_virtual_workspace_root() {
    let workspace = TempDir::new().expect("cargo workspace");
    create_cargo_workspace(workspace.path(), &["only-package"]);

    assert_crates_io_url(workspace.path(), &["--crates-io"], "only-package");
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

fn create_cargo_package(path: &Path, name: &str) {
    fs::create_dir_all(path.join("src")).expect("create package source directory");
    fs::write(
        path.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
    )
    .expect("write Cargo.toml");
    fs::write(path.join("src/lib.rs"), "").expect("write package target");
}

fn assert_crates_io_url(cwd: &Path, args: &[&str], package: &str) {
    assert_printed_url(cwd, args, &format!("https://crates.io/crates/{package}\n"));
}

fn create_cargo_workspace(path: &Path, members: &[&str]) {
    let member_list = members
        .iter()
        .map(|member| format!("\"{member}\""))
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(
        path.join("Cargo.toml"),
        format!("[workspace]\nmembers = [{member_list}]\nresolver = \"3\"\n"),
    )
    .expect("write workspace Cargo.toml");

    for member in members {
        create_cargo_package(&path.join(member), member);
    }
}
