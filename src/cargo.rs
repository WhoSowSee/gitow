use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::Deserialize;

use crate::error::{GitowError, Result};

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    manifest_path: PathBuf,
}

pub fn crates_io_url(cwd: &Path, package_name: Option<&str>) -> Result<String> {
    let package_name = match package_name {
        Some(package_name) => package_name.to_string(),
        None => {
            let manifest_path = locate_manifest(cwd)?;
            let metadata = load_metadata(cwd, &manifest_path)?;
            select_package_name(&metadata.packages, &manifest_path)?
        }
    };
    Ok(format!("https://crates.io/crates/{package_name}"))
}

fn locate_manifest(cwd: &Path) -> Result<PathBuf> {
    cwd.ancestors()
        .map(|directory| directory.join("Cargo.toml"))
        .find(|manifest| manifest.is_file())
        .ok_or(GitowError::NotACargoProject)
}

fn load_metadata(cwd: &Path, manifest_path: &Path) -> Result<Metadata> {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .arg("--manifest-path")
        .arg(manifest_path)
        .current_dir(cwd)
        .output()
        .map_err(GitowError::CargoMetadataCommand)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stderr = if stderr.is_empty() {
            output.status.to_string()
        } else {
            stderr
        };
        return Err(GitowError::CargoMetadataCommandFailed(stderr));
    }

    serde_json::from_slice(&output.stdout).map_err(GitowError::CargoMetadataParse)
}

fn select_package_name(packages: &[Package], manifest_path: &Path) -> Result<String> {
    if let Some(package) = packages
        .iter()
        .find(|package| same_path(&package.manifest_path, manifest_path))
    {
        return Ok(package.name.clone());
    }

    match packages {
        [] => Err(GitowError::NoCargoPackages),
        [package] => Ok(package.name.clone()),
        packages => {
            let mut names = packages
                .iter()
                .map(|package| package.name.as_str())
                .collect::<Vec<_>>();
            names.sort_unstable();
            Err(GitowError::AmbiguousCargoWorkspace {
                packages: names.join(", "),
            })
        }
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}
