use anyhow::{Context, Result};
use std::process::Command;
use tracing::{info, warn};

use crate::launcher::desktop_entry::{collect_entries, parse_exec};
use crate::launcher::search::rank;
use crate::launcher::types::{AppEntry, SearchResult};
use crate::launcher::usage_store::UsageStore;

const DEFAULT_RESULT_LIMIT: usize = 10;

/// Default terminal emulator order of preference.
const TERMINAL_EMULATORS: &[&str] = &[
    "foot", "alacritty", "kitty", "wezterm", "xterm",
];

pub struct LauncherService {
    entries: Vec<AppEntry>,
    usage: UsageStore,
}

impl LauncherService {
    pub fn new() -> Self {
        let entries = collect_entries();
        info!("collected {} desktop entries", entries.len());

        let usage = UsageStore::open().unwrap_or_else(|e| {
            warn!("could not open usage store: {e}; using in-memory fallback");
            // Ephemeral in-memory sled (no persistence) so the service still works.
            let db = sled::Config::new()
                .temporary(true)
                .open()
                .expect("in-memory sled");
            UsageStore { db }
        });

        Self { entries, usage }
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let usage_counts = self.usage.all_counts();
        rank(query, &self.entries, &usage_counts, DEFAULT_RESULT_LIMIT)
    }

    pub fn launch(&self, app_id: &str) -> Result<()> {
        let entry = self
            .entries
            .iter()
            .find(|e| e.id == app_id)
            .with_context(|| format!("unknown app id: {app_id}"))?;

        let (program, args) = parse_exec(&entry.exec, &entry.name)?;

        if entry.terminal {
            self.launch_in_terminal(&program, &args)?;
        } else {
            spawn_detached(&program, &args)?;
        }

        self.usage
            .increment(app_id)
            .unwrap_or_else(|e| warn!("usage store increment failed: {e}"));

        info!("launched {}", entry.name);
        Ok(())
    }

    pub fn run_command(&self, cmd: &str) -> Result<()> {
        let expanded = shellexpand::full(cmd)
            .map(|s| s.into_owned())
            .unwrap_or_else(|_| cmd.to_string());

        let parts: Vec<&str> = expanded.splitn(2, ' ').collect();
        let program = parts[0];
        let args: Vec<&str> = if parts.len() > 1 {
            parts[1].split_whitespace().collect()
        } else {
            vec![]
        };

        spawn_detached(program, &args.iter().map(|s| s.to_string()).collect::<Vec<_>>())?;
        info!("ran command: {cmd}");
        Ok(())
    }

    pub fn refresh(&mut self) {
        self.entries = collect_entries();
        info!("refreshed {} desktop entries", self.entries.len());
    }

    pub fn usage_counts(&self) -> Vec<(String, u32)> {
        self.usage.all_counts()
    }

    fn launch_in_terminal(&self, program: &str, args: &[String]) -> Result<()> {
        let terminal = find_terminal()
            .with_context(|| "no terminal emulator found; install foot, alacritty, or kitty")?;

        let exec_arg = if args.is_empty() {
            program.to_string()
        } else {
            format!("{program} {}", args.join(" "))
        };

        // Most terminals accept "-e <program>" to run a command.
        spawn_detached(&terminal, &["-e".to_string(), exec_arg])
    }
}

/// Spawn `program` with `args` as a detached process (new session, not a child).
fn spawn_detached(program: &str, args: &[String]) -> Result<()> {
    // On Linux we rely on setsid(1) to detach the process from our session so it
    // survives the launcher window closing and is never a direct child.
    #[cfg(target_os = "linux")]
    {
        Command::new("setsid")
            .arg("--fork")
            .arg(program)
            .args(args)
            .spawn()
            .with_context(|| format!("failed to spawn {program} via setsid"))?;
    }

    // Fallback / non-Linux (tests): just spawn directly.
    #[cfg(not(target_os = "linux"))]
    {
        Command::new(program)
            .args(args)
            .spawn()
            .with_context(|| format!("failed to spawn {program}"))?;
    }

    Ok(())
}

fn find_terminal() -> Option<String> {
    TERMINAL_EMULATORS
        .iter()
        .find(|&&t| which::which(t).is_ok())
        .map(|t| t.to_string())
}

// Expose `db` field for test construction in usage_store.rs
#[cfg(test)]
pub(crate) mod test_helpers {
    use super::*;

    pub fn entries_count(svc: &LauncherService) -> usize {
        svc.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_empty_query_returns_results() {
        // This test only verifies the search API; the result set depends on the
        // host machine's installed apps so we just assert it doesn't panic.
        let svc = LauncherService::new();
        let _ = svc.search("");
    }

    #[test]
    fn run_command_bad_program_returns_err() {
        let svc = LauncherService::new();
        // "__nonexistent_binary__" should not be found on any machine.
        let result = svc.run_command("__nonexistent_binary__ --flag");
        assert!(result.is_err());
    }
}
