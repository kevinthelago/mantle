// Private helpers are reachable only through IPC dispatch (compile → load_inner
// → StylingLoader::load → #[tauri::command]), not via a direct Rust call site.
// Rust's dead-code lint doesn't trace through Tauri's runtime dispatch, so we
// suppress it here rather than contort the design.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

use super::error::StyleError;

/// Compile `input` source with the cached `tool` and return the CSS string.
pub fn compile(tool: &str, tool_dir: &Path, input: &str) -> Result<String, StyleError> {
    let tmp = std::env::temp_dir().join("mantle-styling");
    std::fs::create_dir_all(&tmp)?;

    let input_path = tmp.join(format!("input.{}", source_ext(tool)));
    let output_path = tmp.join("output.css");

    std::fs::write(&input_path, input)?;

    let bin = resolve_binary(tool, tool_dir)?;
    execute(&bin, tool, tool_dir, &input_path, &output_path)?;

    let css = std::fs::read_to_string(&output_path)?;

    // Best-effort cleanup; stale files are harmless.
    let _ = std::fs::remove_file(&input_path);
    let _ = std::fs::remove_file(&output_path);

    Ok(css)
}

// ---------------------------------------------------------------------------
// Binary resolution
// ---------------------------------------------------------------------------

enum Binary {
    /// dart <snapshot> — the embedded Dart VM + pre-compiled sass.snapshot.
    DartSnapshot { dart: PathBuf, snapshot: PathBuf },
    /// A Node.js script — requires `node` on PATH (or NODE_PATH env var).
    NodeScript(PathBuf),
}

fn resolve_binary(tool: &str, tool_dir: &Path) -> Result<Binary, StyleError> {
    match tool {
        "sass" => {
            // sass npm package ships bin/sass.js — needs node.
            for rel in &["bin/sass.js", "sass.js"] {
                let p = tool_dir.join(rel);
                if p.exists() {
                    return Ok(Binary::NodeScript(p));
                }
            }
            Err(StyleError::ToolFailed(format!(
                "no sass entry point found in {}",
                tool_dir.display()
            )))
        }

        "sass-embedded" => {
            // Prefer the embedded Dart VM + snapshot (no node needed).
            let dart = dart_vm_path(tool_dir);
            let snapshot = tool_dir.join("dart-sass").join("src").join("sass.snapshot");
            if dart.exists() && snapshot.exists() {
                return Ok(Binary::DartSnapshot { dart, snapshot });
            }
            // Fallback: JS wrapper.
            for rel in &["bin/sass.js", "sass.js"] {
                let p = tool_dir.join(rel);
                if p.exists() {
                    return Ok(Binary::NodeScript(p));
                }
            }
            Err(StyleError::ToolFailed(format!(
                "no sass-embedded binary found in {}",
                tool_dir.display()
            )))
        }

        "tailwindcss" => {
            // tailwindcss ships bin/tailwind.js — needs node.
            for rel in &["bin/tailwind.js", "bin/tailwindcss.js"] {
                let p = tool_dir.join(rel);
                if p.exists() {
                    return Ok(Binary::NodeScript(p));
                }
            }
            Err(StyleError::ToolFailed(format!(
                "no tailwindcss binary found in {}",
                tool_dir.display()
            )))
        }

        other => Err(StyleError::ToolFailed(format!(
            "no runner defined for '{}'",
            other
        ))),
    }
}

fn dart_vm_path(tool_dir: &Path) -> PathBuf {
    let name = if cfg!(target_os = "windows") {
        "dart.exe"
    } else {
        "dart"
    };
    tool_dir.join("dart-sass").join("src").join(name)
}

// ---------------------------------------------------------------------------
// Execution
// ---------------------------------------------------------------------------

fn execute(
    bin: &Binary,
    tool: &str,
    tool_dir: &Path,
    input: &Path,
    output: &Path,
) -> Result<(), StyleError> {
    let result = match bin {
        Binary::DartSnapshot { dart, snapshot } => {
            let mut cmd = sandboxed_cmd(dart);
            cmd.arg(snapshot);
            tool_args(&mut cmd, tool, input, output);
            cmd.output()
        }

        Binary::NodeScript(script) => {
            let node = std::env::var("NODE_PATH").unwrap_or_else(|_| "node".to_string());
            let mut cmd = sandboxed_cmd(Path::new(&node));
            cmd.arg(script);
            // cwd = package root so relative requires in the script resolve.
            cmd.current_dir(tool_dir);
            tool_args(&mut cmd, tool, input, output);
            cmd.output()
        }
    }
    .map_err(|e| StyleError::ToolFailed(e.to_string()))?;

    if !result.status.success() {
        return Err(StyleError::ToolFailed(
            String::from_utf8_lossy(&result.stderr).into_owned(),
        ));
    }
    Ok(())
}

/// A `Command` with a minimal, sanitised environment.
fn sandboxed_cmd(program: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.env_clear();
    // Provide only what tools need to locate system libraries and other binaries.
    for var in &["PATH", "HOME", "USERPROFILE", "TEMP", "TMP"] {
        if let Ok(val) = std::env::var(var) {
            cmd.env(var, val);
        }
    }
    cmd
}

fn tool_args(cmd: &mut Command, tool: &str, input: &Path, output: &Path) {
    match tool {
        "sass" | "sass-embedded" => {
            cmd.arg(input).arg(output).arg("--no-source-map");
        }
        "tailwindcss" => {
            cmd.arg("-i").arg(input).arg("-o").arg(output);
        }
        _ => {
            cmd.arg(input).arg(output);
        }
    }
}

fn source_ext(tool: &str) -> &str {
    match tool {
        "sass" | "sass-embedded" => "scss",
        _ => "css",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_ext_for_sass() {
        assert_eq!(source_ext("sass"), "scss");
        assert_eq!(source_ext("sass-embedded"), "scss");
    }

    #[test]
    fn source_ext_default_css() {
        assert_eq!(source_ext("tailwindcss"), "css");
        assert_eq!(source_ext("postcss"), "css");
    }

    #[test]
    fn unknown_tool_errors() {
        let tmp = std::env::temp_dir();
        assert!(resolve_binary("webpack", &tmp).is_err());
    }
}
