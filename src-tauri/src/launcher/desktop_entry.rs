use anyhow::Result;

use crate::launcher::types::AppEntry;

/// Build the argv for the process spawner.
///
/// Strips XDG field codes (%f, %u, %F, %U, etc.) and expands `~`.
pub fn parse_exec(exec: &str, entry_name: &str) -> Result<(String, Vec<String>)> {
    let expanded = shellexpand::tilde(exec).into_owned();

    let mut parts = shell_words_split(&expanded)?;
    if parts.is_empty() {
        anyhow::bail!("empty Exec field for {entry_name}");
    }

    // Strip single-letter field codes (e.g. %u, %F).
    parts.retain(|p| !(p.starts_with('%') && p.len() == 2));

    let program = parts.remove(0);
    Ok((program, parts))
}

/// Minimal POSIX shell-style word splitting (handles single/double quotes and backslash).
fn shell_words_split(s: &str) -> Result<Vec<String>> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for c in s.chars() {
        if escaped {
            current.push(c);
            escaped = false;
        } else if c == '\\' && !in_single {
            escaped = true;
        } else if c == '\'' && !in_double {
            in_single = !in_single;
        } else if c == '"' && !in_single {
            in_double = !in_double;
        } else if c.is_ascii_whitespace() && !in_single && !in_double {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    Ok(words)
}

// ── Linux-only: XDG desktop entry collection ─────────────────────────────────

#[cfg(target_os = "linux")]
pub fn collect_entries() -> Vec<AppEntry> {
    use freedesktop_desktop_entry::{DesktopEntry, Iter};
    use std::path::PathBuf;

    let locale = current_locale();
    let locale_ref = locale.as_deref();
    let dirs = xdg_application_dirs();

    Iter::new(dirs)
        .filter_map(|path| {
            let bytes = std::fs::read(&path).ok()?;
            let entry = DesktopEntry::decode(&path, &bytes).ok()?;

            if entry.no_display() || entry.hidden() {
                return None;
            }
            if entry.type_().unwrap_or("") != "Application" {
                return None;
            }

            let name = entry.name(locale_ref)?.to_string();
            let exec = entry.exec()?.to_string();
            let id = path.to_string_lossy().into_owned();

            Some(AppEntry {
                id,
                name,
                description: entry.comment(locale_ref).map(|s| s.to_string()),
                icon: entry.icon().map(|s| s.to_string()),
                exec,
                terminal: entry.terminal(),
                categories: split_semicolons(entry.categories().unwrap_or("")),
                keywords: split_semicolons(entry.keywords(locale_ref).unwrap_or("")),
                desktop_file: path.to_string_lossy().into_owned(),
            })
        })
        .collect()
}

#[cfg(not(target_os = "linux"))]
pub fn collect_entries() -> Vec<AppEntry> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn xdg_application_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs::data_local_dir() {
        dirs.push(home.join("applications"));
    }
    let xdg_data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    for dir in xdg_data_dirs.split(':').filter(|s| !s.is_empty()) {
        dirs.push(std::path::PathBuf::from(dir).join("applications"));
    }
    dirs
}

#[cfg(target_os = "linux")]
fn split_semicolons(s: &str) -> Vec<String> {
    s.split(';').filter(|s| !s.is_empty()).map(String::from).collect()
}

#[cfg(target_os = "linux")]
fn current_locale() -> Option<String> {
    std::env::var("LANG")
        .ok()
        .map(|lang| lang.split('.').next().map(|l| l.replace('_', "-")).unwrap_or(lang))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_exec_simple() {
        let (prog, args) = parse_exec("firefox %u", "Firefox").unwrap();
        assert_eq!(prog, "firefox");
        assert!(args.is_empty(), "field code %u should be stripped, got {args:?}");
    }

    #[test]
    fn parse_exec_quoted() {
        let (prog, args) = parse_exec(r#"/usr/bin/env "MY VAR=1" app %f"#, "App").unwrap();
        assert_eq!(prog, "/usr/bin/env");
        assert_eq!(args, vec!["MY VAR=1", "app"]);
    }

    #[test]
    fn parse_exec_multiple_field_codes() {
        let (prog, args) = parse_exec("code --new-window %F", "VS Code").unwrap();
        assert_eq!(prog, "code");
        assert_eq!(args, vec!["--new-window"]);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn parse_exec_home_expansion() {
        unsafe { std::env::set_var("HOME", "/home/testuser") };
        let (prog, _) = parse_exec("~/bin/mytool", "MyTool").unwrap();
        assert_eq!(prog, "/home/testuser/bin/mytool");
    }
}
