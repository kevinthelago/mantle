use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppEntry {
    /// Unique identifier — the desktop file path relative to the XDG data dirs.
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub exec: String,
    pub terminal: bool,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    /// Desktop file path for display/launch purposes.
    pub desktop_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub entry: AppEntry,
    /// Normalised [0, 1] relevance score (higher = better).
    pub score: f32,
    pub launch_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum LaunchTarget {
    App { id: String },
    Command { cmd: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherError {
    pub message: String,
}

impl From<anyhow::Error> for LauncherError {
    fn from(e: anyhow::Error) -> Self {
        Self { message: e.to_string() }
    }
}

impl From<LauncherError> for String {
    fn from(e: LauncherError) -> Self {
        e.message
    }
}
