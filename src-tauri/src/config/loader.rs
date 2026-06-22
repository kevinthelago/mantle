use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use super::schema::Config;

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

    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;

    let config: Config = toml::from_str(&raw)
        .with_context(|| format!("parsing {}", path.display()))?;

    log::info!("loaded config from {}", path.display());
    Ok((config, path))
}

/// Re-read the config file from `path` (used by the hot-reload watcher).
pub fn reload(path: &Path) -> Result<Config> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("re-reading {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("re-parsing {}", path.display()))
}

fn write_defaults(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating config dir {}", parent.display()))?;
    }
    let default = toml::to_string_pretty(&Config::default())
        .context("serialising default config")?;
    let header = "# Mantle config — edit and save; changes apply immediately.\n# Generated on first run.\n\n";
    std::fs::write(path, format!("{header}{default}"))
        .with_context(|| format!("writing {}", path.display()))
}
