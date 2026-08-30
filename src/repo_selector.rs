use std::path::Path;

use crate::{
    error::{GitowError, Result},
    git::{Repository, default_ssh_config_path, looks_like_remote_spec},
    remote::{ParsedRemote, parse_remote_url},
};

const DEFAULT_OWNER_CONFIG: &str = "open.default.owner";

pub fn resolve_repository_specs(specs: &[String], cwd: &Path) -> Result<Vec<ParsedRemote>> {
    let mut context = None;
    specs
        .iter()
        .map(|raw| resolve_repository_spec(raw, cwd, &mut context))
        .collect()
}

fn resolve_repository_spec(
    raw: &str,
    cwd: &Path,
    context: &mut Option<ParsedRemote>,
) -> Result<ParsedRemote> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(GitowError::InvalidRepositorySpec(raw.to_string()));
    }

    if looks_like_remote_spec(raw) {
        return Ok(parse_remote_url(raw, default_ssh_config_path().as_deref()));
    }

    if raw.split('/').any(str::is_empty) {
        return Err(GitowError::InvalidRepositorySpec(raw.to_string()));
    }
    let (prefix, rest) = raw.split_once('/').unwrap_or((raw, ""));

    if let Some(domain) = forge_domain(prefix) {
        return if rest.is_empty() {
            Err(GitowError::InvalidRepositorySpec(raw.to_string()))
        } else if rest.contains('/') {
            Ok(parse_remote_url(&format!("https://{domain}/{rest}"), None))
        } else {
            resolve_inherited_repository(Some(domain), rest, cwd, context)
        };
    }

    if prefix.contains('.') {
        if !rest.contains('/') {
            return Err(GitowError::InvalidRepositorySpec(raw.to_string()));
        }
        return Ok(parse_remote_url(&format!("https://{raw}"), None));
    }

    if rest.is_empty() {
        return resolve_inherited_repository(None, raw, cwd, context);
    }

    Ok(parse_remote_url(&format!("https://github.com/{raw}"), None))
}

fn resolve_inherited_repository(
    domain: Option<&str>,
    repository: &str,
    cwd: &Path,
    context: &mut Option<ParsedRemote>,
) -> Result<ParsedRemote> {
    if context.is_none() {
        *context = Some(inherited_context(cwd)?);
    }
    let context = context.as_ref().ok_or(GitowError::MissingRepositoryOwner)?;
    let path = format!("{}/{repository}", context.url_path);
    if let Some(domain) = domain {
        Ok(parse_remote_url(&format!("https://{domain}/{path}"), None))
    } else {
        let mut remote = context.clone();
        remote.url_path = path;
        Ok(remote)
    }
}

fn inherited_context(cwd: &Path) -> Result<ParsedRemote> {
    let repository = Repository::new(cwd);
    if let Some(namespace) = repository
        .scoped_config("--global", DEFAULT_OWNER_CONFIG)?
        .and_then(usable_namespace)
    {
        return Ok(parse_remote_url(
            &format!("https://github.com/{namespace}"),
            None,
        ));
    }

    match repository.ensure_work_tree() {
        Ok(()) => {}
        Err(GitowError::NotAGitRepository) => return Err(GitowError::MissingRepositoryOwner),
        Err(error) => return Err(error),
    }

    if let Some(namespace) = repository
        .scoped_config("--local", DEFAULT_OWNER_CONFIG)?
        .and_then(usable_namespace)
    {
        return Ok(parse_remote_url(
            &format!("https://github.com/{namespace}"),
            None,
        ));
    }

    let origin = match repository.configured_remote("origin") {
        Ok(origin) => origin,
        Err(GitowError::MissingRemote(_)) => return Err(GitowError::MissingRepositoryOwner),
        Err(error) => return Err(error),
    };
    let mut origin = origin;
    origin.url_path = origin
        .url_path
        .rsplit_once('/')
        .map(|(namespace, _)| namespace)
        .filter(|namespace| !namespace.is_empty())
        .ok_or(GitowError::MissingRepositoryOwner)?
        .to_string();
    Ok(origin)
}

fn usable_namespace(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && !value.chars().any(char::is_whitespace)).then(|| value.to_string())
}

fn forge_domain(alias: &str) -> Option<&'static str> {
    match alias.to_ascii_lowercase().as_str() {
        "github" => Some("github.com"),
        "gitlab" => Some("gitlab.com"),
        "codeberg" => Some("codeberg.org"),
        "bitbucket" => Some("bitbucket.org"),
        "gitea" => Some("gitea.com"),
        "azure" => Some("dev.azure.com"),
        _ => None,
    }
}
