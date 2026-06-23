use std::path::{Component, Path, PathBuf};

use flate2::read::GzDecoder;
use tar::Archive;

use super::error::StyleError;

/// `$XDG_CACHE_HOME/mantle/tooling` (or the OS-equivalent base).
pub fn base_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cache")
        })
        .join("mantle")
        .join("tooling")
}

/// Content-addressed cache path for a single tool version.
///
/// Scoped package names (e.g. `@scope/name`) have the `@` and `/` replaced so
/// the result is safe as a directory name on every supported OS.
pub fn tool_dir(pkg: &str, version: &str) -> PathBuf {
    let safe = pkg.replace('@', "_at_").replace('/', "_");
    base_cache_dir().join(format!("{}@{}", safe, version))
}

/// A cache entry is complete when the `.ok` sentinel file is present.
pub fn is_cached(pkg: &str, version: &str) -> bool {
    let dir = tool_dir(pkg, version);
    dir.exists() && dir.join(".ok").exists()
}

/// Extract a `.tgz` npm tarball to `dest`, stripping the `package/` root prefix
/// all npm tarballs include.
///
/// Path traversal components (`..`) and absolute paths are silently skipped.
pub fn extract_tarball(tarball: &[u8], dest: &Path) -> Result<(), StyleError> {
    std::fs::create_dir_all(dest)?;

    let gz = GzDecoder::new(tarball);
    let mut archive = Archive::new(gz);
    archive.set_overwrite(true);
    archive.set_preserve_permissions(false);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let raw = entry.path()?.into_owned();

        // Strip the mandatory "package/" npm prefix.
        let rel = raw
            .strip_prefix("package")
            .unwrap_or(&raw)
            .to_path_buf();

        // Reject traversal and absolute paths.
        if rel.is_absolute()
            || rel
                .components()
                .any(|c| c == Component::ParentDir)
        {
            continue;
        }

        if rel == Path::new("") {
            continue;
        }

        let out = dest.join(&rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        entry.unpack(&out)?;
    }

    // Write sentinel only after a fully successful extraction.
    std::fs::write(dest.join(".ok"), "")?;
    Ok(())
}

/// Path of the persisted last-good CSS file.
pub fn last_good_path() -> PathBuf {
    base_cache_dir().join("last-good.css")
}

pub fn read_last_good() -> Option<String> {
    std::fs::read_to_string(last_good_path()).ok()
}

pub fn write_last_good(css: &str) -> std::io::Result<()> {
    let path = last_good_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, css)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_dir_contains_mantle() {
        assert!(base_cache_dir().to_string_lossy().contains("mantle"));
    }

    #[test]
    fn tool_dir_encodes_pkg_and_version() {
        let d = tool_dir("sass", "1.69.5");
        assert!(d.to_string_lossy().ends_with("sass@1.69.5"));
    }

    #[test]
    fn scoped_package_name_is_safe() {
        let d = tool_dir("@scope/pkg", "1.0.0");
        let filename = d.file_name().unwrap().to_string_lossy();
        // The scoped '@' prefix must be rewritten so the dir name is FS-safe.
        assert!(!filename.starts_with('@'), "filename starts with raw '@'");
        // Slashes must not appear inside a single path component.
        assert!(!filename.contains('/'), "filename contains '/'");
        assert!(!filename.contains('\\'), "filename contains '\\'");
    }
}
