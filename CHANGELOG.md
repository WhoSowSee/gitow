# Changelog

## [2.0.0] - 2026-08-28

### Breaking Changes

- Changed: default remote selection now prefers `origin` before the current branch's tracked remote

### Features

- Added: `-v` and `--version` flags

### Internal

- Simplified: browser, remote, and provider code while consolidating integration test helpers and removing redundant coverage
- Added: CI and release workflows with cross-platform archives, Snap, Nix, package verification, and GitHub Releases

### Maintenance

- Reduced: dependency feature sets to only those used by gitow
- Updated: Rust dependencies to their current compatible releases
- Lowered: MSRV to Rust 1.88 and replaced the package include list with explicit exclusions
- Fixed: CI now runs only from tag-triggered or manually dispatched releases and installs actionlint from its checksum-verified release archive

## [1.0.0] - 2026-08-28

### Added

- Cross-platform CLI for opening repository roots, branches, commits, issues, pull requests, commit history, releases, tracked files, and custom URL suffixes
- Support for GitHub, GitLab, Gitea and Codeberg, Bitbucket Cloud and Server, Azure DevOps, AWS CodeCommit, GitHub Gists, and generic remotes
- Remote resolution through Git configuration, `insteadOf` rewrites, SSH aliases, and forge, domain, and protocol overrides
- Branch-aware links using upstream branches, current branches, exact tags, and commit SHAs, plus multi-remote support
- Browser launching on Windows, macOS, Linux, and WSL, with script-friendly output through `--print` and `BROWSER=echo`
- Unit and integration test coverage for CLI behavior and provider URL generation
