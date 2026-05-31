use std::fmt::Write as _;
use std::path::PathBuf;

const ALLOWED_KEYS: &[&str] = &[
    "GROK_SSO_COOKIE",
    "GROK_SSO_RW_COOKIE",
    "GROK_EXTRA_COOKIES",
    "TOKEN_PROVIDER_URL",
    "CHALLENGE_HEADER_HEX",
    "CHALLENGE_SUFFIX",
    "CHALLENGE_TRAILER",
    "API_KEY",
    "HOST",
    "PORT",
    "LOG_LEVEL",
    "SESSION_CHECK_INTERVAL_SECS",
    "GROK_BASE_URL",
];

pub fn run(args: &[String]) -> Result<(), String> {
    let mut updates: Vec<(String, String)> = Vec::with_capacity(args.len());
    for arg in args {
        let Some((key, value)) = arg.split_once('=') else {
            return Err(format!("argument '{arg}' is not KEY=VALUE"));
        };
        let key = key.trim();
        if !ALLOWED_KEYS.contains(&key) {
            return Err(format!(
                "unknown key '{key}'\nallowed: {}",
                ALLOWED_KEYS.join(", ")
            ));
        }
        updates.push((key.to_owned(), unquote(value.trim())));
    }
    if updates.is_empty() {
        return Err("usage: grok-server update-keys KEY=VALUE [KEY=VALUE ...]".into());
    }

    let path = env_path();
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    std::fs::write(&path, merge(&existing, &updates))
        .map_err(|e| format!("failed to write {}: {e}", path.display()))?;

    for (key, _) in &updates {
        println!("set {key}");
    }
    println!("wrote {}", path.display());
    Ok(())
}

fn merge(existing: &str, updates: &[(String, String)]) -> String {
    let mut written = vec![false; updates.len()];
    let mut out = String::with_capacity(existing.len() + 256);
    for line in existing.lines() {
        match updates.iter().position(|(key, _)| defines(line, key)) {
            Some(i) => {
                let (key, value) = &updates[i];
                let _ = writeln!(out, "{}", render(key, value));
                written[i] = true;
            }
            None => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    for (i, (key, value)) in updates.iter().enumerate() {
        if !written[i] {
            let _ = writeln!(out, "{}", render(key, value));
        }
    }
    out
}

fn defines(line: &str, key: &str) -> bool {
    line.trim_start()
        .strip_prefix(key)
        .is_some_and(|rest| rest.trim_start().starts_with('='))
}

fn render(key: &str, value: &str) -> String {
    if key == "CHALLENGE_TRAILER" && !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit())
    {
        return format!("{key}={value}");
    }
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("{key}=\"{escaped}\"")
}

fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    let quoted = bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''));
    if quoted {
        value[1..value.len() - 1].to_owned()
    } else {
        value.to_owned()
    }
}

fn env_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir = cwd.as_path();
    loop {
        let candidate = dir.join(".env");
        if candidate.is_file() {
            return candidate;
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    cwd.join(".env")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_existing_key_preserving_others() {
        let env = "# comment\nGROK_SSO_COOKIE=old\nPORT=3000\n";
        let out = merge(env, &[("GROK_SSO_COOKIE".into(), "new".into())]);
        assert!(out.contains("# comment"));
        assert!(out.contains("GROK_SSO_COOKIE=\"new\""));
        assert!(out.contains("PORT=3000"));
        assert!(!out.contains("=old"));
    }

    #[test]
    fn appends_missing_key() {
        let out = merge("PORT=3000\n", &[("CHALLENGE_SUFFIX".into(), "!x".into())]);
        assert!(out.contains("PORT=3000"));
        assert!(out.trim_end().ends_with("CHALLENGE_SUFFIX=\"!x\""));
    }

    #[test]
    fn trailer_written_unquoted_when_numeric() {
        assert_eq!(render("CHALLENGE_TRAILER", "3"), "CHALLENGE_TRAILER=3");
    }

    #[test]
    fn values_with_quotes_are_escaped() {
        assert_eq!(
            render("CHALLENGE_SUFFIX", "a\"b\\c"),
            "CHALLENGE_SUFFIX=\"a\\\"b\\\\c\""
        );
    }

    #[test]
    fn defines_matches_only_exact_key() {
        assert!(defines("  PORT = 1", "PORT"));
        assert!(!defines("PORTX=1", "PORT"));
        assert!(!defines("API_KEYS=1", "API_KEY"));
    }

    #[test]
    fn unquote_strips_matching_quotes() {
        assert_eq!(unquote("\"x\""), "x");
        assert_eq!(unquote("'x'"), "x");
        assert_eq!(unquote("x"), "x");
    }
}
