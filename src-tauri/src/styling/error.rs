use thiserror::Error;

#[derive(Debug, Error)]
pub enum StyleError {
    #[error("package '{0}' is not in the allowlist")]
    NotAllowed(String),

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("integrity mismatch — expected {expected}, got {actual}")]
    IntegrityMismatch { expected: String, actual: String },

    #[error("no sha512 integrity hash in registry metadata: {0}")]
    UnsupportedIntegrity(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("tool execution failed: {0}")]
    ToolFailed(String),

    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
}

// Allow StyleError to be returned from Tauri commands.
impl serde::Serialize for StyleError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
