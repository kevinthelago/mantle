use super::schema::Config;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const CONFIG_DIR: &str = "mantle";
const CONFIG_FILE: &str = "config.toml";

/// Returns the XDG-compliant config file path: `$XDG_CONFIG_HOME/mantle/config.toml`
/// falling back to `~/.config/mantle/config.toml`.
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join(CONFIG_DIR)
        .join(CONFIG_FILE)
}

/// Load config from disk, writing defaults on first run.
/// Returns `(config, path_that_was_loaded)`.
pub fn load() -> Result<(Config, PathBuf)> {
    let path = config_path();

    if !path.exists() {
        write_defaults(&path).context("writing default config")?;
        log::info!("created default config at {}", path.display());
        return Ok((Config::default(), path));
    }

    let raw =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;

    let config: Config =
        toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;

    log::info!("loaded config from {}", path.display());
    Ok((config, path))
}

/// Re-read the config file from `path` (used by the hot-reload watcher).
pub fn reload(path: &Path) -> Result<Config> {
    let raw =
        std::fs::read_to_string(path).with_context(|| format!("re-reading {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("re-parsing {}", path.display()))
}

fn write_defaults(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating config dir {}", parent.display()))?;
    }
    let default =
        toml::to_string_pretty(&Config::default()).context("serialising default config")?;
    let header = "# Mantle config — edit and save; changes apply immediately.\n# Generated on first run.\n\n";
    std::fs::write(path, format!("{header}{default}"))
        .with_context(|| format!("writing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_config(dir: &TempDir) -> PathBuf {
        dir.path().join("config.toml")
    }

    #[test]
    fn config_path_is_non_empty() {
        let p = config_path();
        assert!(!p.as_os_str().is_empty());
        assert!(p.to_string_lossy().contains("mantle"));
        assert!(p.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn write_defaults_creates_valid_toml() {
        let dir = TempDir::new().unwrap();
        let path = temp_config(&dir);
        write_defaults(&path).expect("write_defaults failed");
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: Config =
            toml::from_str(&content).expect("invalid TOML produced by write_defaults");
        assert_eq!(parsed.clock.format, "%H:%M");
    }

    #[test]
    fn write_defaults_includes_header_comment() {
        let dir = TempDir::new().unwrap();
        let path = temp_config(&dir);
        write_defaults(&path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.starts_with('#'), "config file should start with a comment");
    }

    #[test]
    fn write_defaults_then_reload_round_trips() {
        let dir = TempDir::new().unwrap();
        let path = temp_config(&dir);
        write_defaults(&path).unwrap();
        let cfg = reload(&path).expect("reload after write_defaults failed");
        assert_eq!(cfg.clock.format, "%H:%M");
        assert_eq!(cfg.outputs.len(), 1);
        assert_eq!(cfg.outputs[0].name, "*");
    }

    #[test]
    fn reload_parses_a_valid_config_file() {
        let dir = TempDir::new().unwrap();
        let path = temp_config(&dir);
        let toml_content = r#"
[clock]
format = "%I:%M %p"
interval = 500
"#;
        std::fs::write(&path, toml_content).unwrap();
        let cfg = reload(&path).expect("reload failed");
        assert_eq!(cfg.clock.format, "%I:%M %p");
        assert_eq!(cfg.clock.interval, 500);
    }

    #[test]
    fn reload_errors_on_invalid_toml() {
        let dir = TempDir::new().unwrap();
        let path = temp_config(&dir);
        std::fs::write(&path, "this is not valid toml }{").unwrap();
        assert!(reload(&path).is_err());
    }

    #[test]
    fn reload_errors_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("does-not-exist.toml");
        assert!(reload(&path).is_err());
    }
}
