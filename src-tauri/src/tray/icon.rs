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

/// Convert network-byte-order ARGB32 to a PNG byte stream.
fn argb_to_png(width: u32, height: u32, argb: &[u8]) -> Option<Vec<u8>> {
    use std::io::Write;

    let expected = (width * height * 4) as usize;
    if argb.len() < expected {
        return None;
    }
    // Convert ARGB → RGBA (PNG expects RGBA).
    let mut rgba = Vec::with_capacity(expected);
    for chunk in argb[..expected].chunks_exact(4) {
        let a = chunk[0];
        let r = chunk[1];
        let g = chunk[2];
        let b = chunk[3];
        rgba.extend_from_slice(&[r, g, b, a]);
    }

    // Encode as PNG manually (tiny inline encoder).
    let mut png = Vec::new();
    write_png(&mut png, width, height, &rgba).ok()?;
    Some(png)
}

/// Minimal PNG encoder (no external dep) — IHDR + IDAT (uncompressed) + IEND.
fn write_png(out: &mut Vec<u8>, w: u32, h: u32, rgba: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    // PNG signature
    out.write_all(b"\x89PNG\r\n\x1a\n")?;

    // IHDR
    let ihdr_data: Vec<u8> = {
        let mut d = Vec::new();
        d.extend_from_slice(&w.to_be_bytes());
        d.extend_from_slice(&h.to_be_bytes());
        d.push(8); // bit depth
        d.push(6); // RGBA
        d.push(0); // compression
        d.push(0); // filter
        d.push(0); // interlace
        d
    };
    write_chunk(out, b"IHDR", &ihdr_data)?;

    // IDAT — filter type 0 (None) per scanline, then zlib-compress.
    let mut raw = Vec::new();
    for row in 0..h as usize {
        raw.push(0u8); // filter type None
        raw.extend_from_slice(&rgba[row * w as usize * 4..(row + 1) * w as usize * 4]);
    }
    let compressed = miniz_compress(&raw);
    write_chunk(out, b"IDAT", &compressed)?;

    // IEND
    write_chunk(out, b"IEND", &[])?;
    Ok(())
}

fn write_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    out.write_all(&(data.len() as u32).to_be_bytes())?;
    out.write_all(tag)?;
    out.write_all(data)?;
    let crc = crc32(tag, data);
    out.write_all(&crc.to_be_bytes())?;
    Ok(())
}

fn crc32(tag: &[u8], data: &[u8]) -> u32 {
    // IEEE polynomial
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let mut t = [0u32; 256];
        for n in 0..256usize {
            let mut c = n as u32;
            for _ in 0..8 {
                if c & 1 != 0 {
                    c = 0xEDB88320 ^ (c >> 1);
                } else {
                    c >>= 1;
                }
            }
            t[n] = c;
        }
        t
    });
    let mut crc = 0xFFFFFFFFu32;
    for &b in tag.iter().chain(data.iter()) {
        crc = table[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFFFFFF
}

/// Minimal zlib (RFC 1950) wrapper around deflate level-0 (stored blocks).
fn miniz_compress(data: &[u8]) -> Vec<u8> {
    // zlib header: CMF=0x78, FLG=0x01 (no dict, check bits)
    let mut out = vec![0x78u8, 0x01];
    // deflate stored blocks (BTYPE=00)
    const BLOCK: usize = 65535;
    let mut pos = 0;
    while pos < data.len() {
        let end = (pos + BLOCK).min(data.len());
        let is_last = end == data.len();
        out.push(if is_last { 1 } else { 0 }); // BFINAL + BTYPE=00
        let len = (end - pos) as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(&data[pos..end]);
        pos = end;
    }
    if data.is_empty() {
        out.extend_from_slice(&[1u8, 0, 0, 0xFF, 0xFF]);
    }
    // Adler-32 checksum
    let mut s1 = 1u32;
    let mut s2 = 0u32;
    for &b in data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    out.extend_from_slice(&((s2 << 16) | s1).to_be_bytes());
    out
}
