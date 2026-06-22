//! Icon resolution: freedesktop theme lookup + ARGB pixmap → PNG base64.

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use std::path::{Path, PathBuf};

/// Resolve an icon to a data URI or an absolute file:// URL the frontend can use.
///
/// Priority:
///   1. IconThemePath + IconName  (app-supplied theme directory)
///   2. System icon theme lookup  (hicolor fallback)
///   3. IconPixmap first element  → inline PNG data URI
pub fn resolve_icon(
    icon_name: Option<&str>,
    icon_theme_path: Option<&str>,
    icon_pixmap: Option<&[(i32, i32, Vec<u8>)]>,
    size_hint: u32,
) -> Option<String> {
    if let Some(name) = icon_name.filter(|n| !n.is_empty()) {
        // 1. Try the app-supplied theme directory first.
        if let Some(theme_path) = icon_theme_path.filter(|p| !p.is_empty()) {
            if let Some(path) = lookup_in_dir(Path::new(theme_path), name, size_hint) {
                return Some(file_url(&path));
            }
        }
        // 2. Freedesktop theme search.
        if let Some(path) = lookup_freedesktop(name, size_hint) {
            return Some(file_url(&path));
        }
    }
    // 3. Pixmap fallback.
    if let Some(pixmaps) = icon_pixmap {
        if let Some(data_uri) = best_pixmap(pixmaps, size_hint) {
            return Some(data_uri);
        }
    }
    None
}

fn file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

/// Look for `name.{png,svg,xpm}` in a flat directory (app-supplied theme paths
/// are usually non-standard flat directories).
fn lookup_in_dir(dir: &Path, name: &str, _size: u32) -> Option<PathBuf> {
    for ext in &["png", "svg", "xpm"] {
        let p = dir.join(format!("{name}.{ext}"));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Very lightweight freedesktop icon-theme lookup.
/// Searches hicolor + Adwaita in the standard XDG dirs, preferring sizes ≥ size_hint.
fn lookup_freedesktop(name: &str, size_hint: u32) -> Option<PathBuf> {
    let search_paths = icon_search_paths();
    let themes = ["hicolor", "Adwaita", "gnome"];
    let preferred_sizes: &[&str] = &["scalable", "symbolic", "48x48", "32x32", "24x24", "22x22", "16x16"];

    for base in &search_paths {
        for theme in themes {
            let theme_dir = base.join(theme);
            for size_dir in preferred_sizes {
                let size_u = size_dir.split('x').next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
                if size_u > 0 && size_u < size_hint / 2 {
                    continue;
                }
                for cat in &["apps", "devices", "status", "actions", "places"] {
                    for ext in &["svg", "png", "xpm"] {
                        let p = theme_dir.join(size_dir).join(cat).join(format!("{name}.{ext}"));
                        if p.exists() {
                            return Some(p);
                        }
                    }
                }
            }
        }
        // Also try pixmaps.
        for ext in &["png", "svg", "xpm"] {
            let p = base.parent().unwrap_or(base).join("pixmaps").join(format!("{name}.{ext}"));
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}

fn icon_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(home).join(".local/share/icons"));
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg.split(':') {
            paths.push(PathBuf::from(dir).join("icons"));
        }
    }
    paths.push(PathBuf::from("/usr/share/icons"));
    paths
}

/// Pick the largest pixmap ≤ size_hint (or smallest if all are larger) and
/// encode it as an inline PNG data URI.
fn best_pixmap(pixmaps: &[(i32, i32, Vec<u8>)], size_hint: u32) -> Option<String> {
    if pixmaps.is_empty() {
        return None;
    }
    let target = size_hint as i32;
    // Choose pixmap with size closest to (and ≥) target, else the largest one.
    let chosen = pixmaps
        .iter()
        .filter(|(w, h, _)| *w >= target && *h >= target)
        .min_by_key(|(w, h, _)| w + h)
        .or_else(|| pixmaps.iter().max_by_key(|(w, h, _)| w + h))?;

    let (w, h, argb) = chosen;
    let png = argb_to_png(*w as u32, *h as u32, argb)?;
    let b64 = B64.encode(&png);
    Some(format!("data:image/png;base64,{b64}"))
}

/// Convert network-byte-order ARGB32 to a PNG byte stream using the `image` crate.
fn argb_to_png(width: u32, height: u32, argb: &[u8]) -> Option<Vec<u8>> {
    let expected = (width * height * 4) as usize;
    if argb.len() < expected {
        return None;
    }
    // Network-byte-order ARGB → RGBA
    let mut rgba = Vec::with_capacity(expected);
    for chunk in argb[..expected].chunks_exact(4) {
        rgba.push(chunk[1]); // R
        rgba.push(chunk[2]); // G
        rgba.push(chunk[3]); // B
        rgba.push(chunk[0]); // A
    }
    let img = image::RgbaImage::from_raw(width, height, rgba)?;
    let mut out = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .ok()?;
    Some(out)
}
