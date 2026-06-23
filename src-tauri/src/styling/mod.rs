mod allowlist;
mod cache;
pub mod error;
mod registry;
mod runner;

pub use error::StyleError;

use std::sync::Arc;

use serde::Serialize;
use tokio::sync::Mutex;

/// Result returned to the frontend after a successful compile.
#[derive(Debug, Serialize, specta::Type)]
pub struct LoadResult {
    /// Compiled CSS.
    pub css: String,
    /// `true` when the response is from the last-good fallback (i.e. the live
    /// compile failed).
    pub from_cache: bool,
}

/// Download, verify, cache, and execute npm styling tools on demand.
///
/// Stored as Tauri managed state; all async operations are serialised through
/// `load_lock` so concurrent invocations don't stomp each other's temp files.
pub struct StylingLoader {
    client: reqwest::Client,
    load_lock: Arc<Mutex<()>>,
}

impl StylingLoader {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("mantle-styling-loader/0.1")
            .build()
            .expect("failed to build reqwest client");

        Self {
            client,
            load_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Compile `input` with `tool@version` and return the CSS.
    ///
    /// On any error, returns the last-good CSS (if any) so the shell never
    /// breaks due to a styling failure.
    pub async fn load(
        &self,
        tool: &str,
        version: &str,
        input: &str,
    ) -> Result<LoadResult, StyleError> {
        if !allowlist::is_allowed(tool) {
            return Err(StyleError::NotAllowed(tool.to_string()));
        }

        let _guard = self.load_lock.lock().await;

        match self.load_inner(tool, version, input).await {
            Ok(css) => {
                let _ = cache::write_last_good(&css);
                Ok(LoadResult {
                    css,
                    from_cache: false,
                })
            }
            Err(err) => {
                if let Some(css) = cache::read_last_good() {
                    eprintln!(
                        "[styling-loader] compile failed (serving last-good): {}",
                        err
                    );
                    Ok(LoadResult {
                        css,
                        from_cache: true,
                    })
                } else {
                    Err(err)
                }
            }
        }
    }

    async fn load_inner(
        &self,
        tool: &str,
        version: &str,
        input: &str,
    ) -> Result<String, StyleError> {
        let tool_dir = cache::tool_dir(tool, version);

        if !cache::is_cached(tool, version) {
            let tarball = registry::fetch_tarball(&self.client, tool, version).await?;
            cache::extract_tarball(&tarball, &tool_dir)?;
        }

        runner::compile(tool, &tool_dir, input)
    }
}

impl Default for StylingLoader {
    fn default() -> Self {
        Self::new()
    }
}
