use std::{fs, path::Path};

pub fn resolve_alias(path: &Path, alias: &str) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let mut current_match = false;
    let mut resolved = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let keyword = match parts.next() {
            Some(keyword) => keyword,
            None => continue,
        };

        if keyword.eq_ignore_ascii_case("Host") {
            current_match = parts.any(|pattern| wildcard_matches(pattern, alias));
            continue;
        }

        if current_match
            && keyword.eq_ignore_ascii_case("HostName")
            && let Some(hostname) = parts.next()
        {
            resolved = Some(hostname.replace("%h", alias));
        }
    }

    resolved
}

fn wildcard_matches(pattern: &str, candidate: &str) -> bool {
    let pattern = pattern.as_bytes();
    let candidate = candidate.as_bytes();
    let mut pattern_index = 0usize;
    let mut candidate_index = 0usize;
    let mut wildcard_index = None;
    let mut backtrack_index = 0usize;

    while candidate_index < candidate.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == b'?'
                || pattern[pattern_index] == candidate[candidate_index])
        {
            pattern_index += 1;
            candidate_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            wildcard_index = Some(pattern_index);
            pattern_index += 1;
            backtrack_index = candidate_index;
        } else if let Some(wildcard) = wildcard_index {
            pattern_index = wildcard + 1;
            backtrack_index += 1;
            candidate_index = backtrack_index;
        } else {
            return false;
        }
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }

    pattern_index == pattern.len()
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::resolve_alias;

    fn write_ssh_config(contents: &str) -> NamedTempFile {
        let file = NamedTempFile::new().expect("temporary file");
        std::fs::write(file.path(), contents).expect("write ssh config");
        file
    }

    #[test]
    fn resolves_simple_alias_and_hostname_case_insensitively() {
        let file = write_ssh_config(
            "\
            host basic\n\
              hOsTnAmE basic.example.com\n\
            ",
        );

        assert_eq!(
            resolve_alias(file.path(), "basic"),
            Some("basic.example.com".to_string())
        );
    }

    #[test]
    fn resolves_later_rules_as_overrides() {
        let file = write_ssh_config(
            "\
            Host zero*\n\
              HostName zero.example.com\n\
            Host zero-override\n\
              HostName override.example.com\n\
            ",
        );

        assert_eq!(
            resolve_alias(file.path(), "zero-override"),
            Some("override.example.com".to_string())
        );
    }

    #[test]
    fn supports_wildcards_and_host_substitution() {
        let file = write_ssh_config(
            "\
            Host subone?\n\
              HostName %h.example.com\n\
            Host zero*\n\
              HostName zero.example.com\n\
            ",
        );

        assert_eq!(
            resolve_alias(file.path(), "subone7"),
            Some("subone7.example.com".to_string())
        );
        assert_eq!(
            resolve_alias(file.path(), "zero-dev"),
            Some("zero.example.com".to_string())
        );
        assert_eq!(resolve_alias(file.path(), "subone"), None);
    }
}
