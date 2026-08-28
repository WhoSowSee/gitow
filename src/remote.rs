use crate::ssh_config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRemote {
    pub domain: String,
    pub url_path: String,
    pub protocol: String,
    pub forge: Option<String>,
    pub config_base_url: String,
}

impl ParsedRemote {
    pub fn with_overrides(
        mut self,
        domain: Option<String>,
        protocol: Option<String>,
        forge: Option<String>,
    ) -> Self {
        if let Some(domain) = domain {
            self.domain = domain;
        }
        if let Some(protocol) = protocol {
            self.protocol = protocol;
        }
        self.forge = forge
            .as_deref()
            .map(normalize_forge_name)
            .or_else(|| self.forge.clone())
            .or_else(|| detect_forge_family(&self.domain));
        self
    }
}

pub fn parse_remote_url(raw: &str, ssh_config_path: Option<&std::path::Path>) -> ParsedRemote {
    if let Some((git_protocol, remainder)) = split_scheme(raw) {
        parse_url_style_remote(git_protocol, remainder)
    } else {
        parse_scp_style_remote(raw, ssh_config_path)
    }
}

fn parse_url_style_remote(git_protocol: &str, remainder: &str) -> ParsedRemote {
    let uri_without_user = remainder
        .rsplit_once('@')
        .map_or(remainder, |(_, rest)| rest);
    let (mut domain, url_path) = split_once(uri_without_user, '/');

    if git_protocol != "http"
        && git_protocol != "https"
        && let Some((host, _)) = domain.rsplit_once(':')
    {
        domain = host.to_string();
    }

    let protocol = if git_protocol == "http" {
        "http"
    } else {
        "https"
    };

    build_parsed_remote(domain, url_path, protocol)
}

fn parse_scp_style_remote(raw: &str, ssh_config_path: Option<&std::path::Path>) -> ParsedRemote {
    let uri_without_user = raw.rsplit_once('@').map_or(raw, |(_, rest)| rest);
    let (domain, url_path) = split_once(uri_without_user, ':');
    let domain = ssh_config_path
        .and_then(|path| ssh_config::resolve_alias(path, &domain))
        .unwrap_or(domain);
    build_parsed_remote(domain, url_path, "https")
}

fn build_parsed_remote(domain: String, url_path: String, protocol: &str) -> ParsedRemote {
    let protocol = protocol.to_string();
    let url_path = normalize_url_path(&url_path);
    let forge = detect_forge_family(&domain);
    let config_base_url = format!("{protocol}://{domain}");

    ParsedRemote {
        domain,
        url_path,
        protocol,
        forge,
        config_base_url,
    }
}

fn detect_forge_family(domain: &str) -> Option<String> {
    let normalized = domain.to_ascii_lowercase();

    if normalized == "codeberg.org"
        || normalized.ends_with(".codeberg.org")
        || normalized.contains("forgejo")
        || normalized.contains("gitea")
        || normalized.contains("gogs")
    {
        return Some("gitea".to_string());
    }

    None
}

fn normalize_forge_name(forge: &str) -> String {
    match forge.to_ascii_lowercase().as_str() {
        "forgejo" | "gogs" | "gitea" => "gitea".to_string(),
        other => other.to_string(),
    }
}

fn split_scheme(raw: &str) -> Option<(&str, &str)> {
    raw.find("://")
        .map(|index| (&raw[..index], &raw[index + 3..]))
}

fn split_once(input: &str, separator: char) -> (String, String) {
    input
        .split_once(separator)
        .map(|(left, right)| (left.to_string(), right.to_string()))
        .unwrap_or_else(|| (input.to_string(), String::new()))
}

fn normalize_url_path(path: &str) -> String {
    path.trim_start_matches('/')
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::parse_remote_url;

    #[test]
    fn parses_https_remote_and_preserves_http_scheme() {
        let parsed = parse_remote_url("http://github.com/user/repo.git", None);

        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "github.com");
        assert_eq!(parsed.url_path, "user/repo");
        assert_eq!(parsed.config_base_url, "http://github.com");
    }
}
