use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha512};

use super::error::StyleError;

#[derive(Debug, Deserialize)]
struct DistInfo {
    tarball: String,
    integrity: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VersionDoc {
    dist: DistInfo,
}

/// Fetch a package tarball from the npm registry and verify its sha512 integrity.
///
/// Returns the raw tarball bytes on success; aborts without writing anything on
/// any fetch, status, or integrity error.
pub async fn fetch_tarball(
    client: &Client,
    pkg: &str,
    version: &str,
) -> Result<Vec<u8>, StyleError> {
    let url = format!("https://registry.npmjs.org/{}/{}", pkg, version);
    let doc: VersionDoc = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let integrity = doc.dist.integrity.ok_or_else(|| {
        StyleError::UnsupportedIntegrity(format!(
            "no integrity field for {}@{}",
            pkg, version
        ))
    })?;

    let bytes = client
        .get(&doc.dist.tarball)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();

    verify_sha512(&integrity, &bytes)?;

    Ok(bytes)
}

/// Verify an SRI integrity string against `data`.
///
/// `integrity` may be a space-separated list of SRI hashes (as the npm registry
/// sometimes produces). We require at least one sha512 entry.
fn verify_sha512(integrity: &str, data: &[u8]) -> Result<(), StyleError> {
    let hash_entry = integrity
        .split_whitespace()
        .find(|s| s.starts_with("sha512-"))
        .ok_or_else(|| StyleError::UnsupportedIntegrity(integrity.to_string()))?;

    let b64 = hash_entry.strip_prefix("sha512-").unwrap();
    let expected = STANDARD.decode(b64)?;
    let actual = Sha512::digest(data);

    if actual.as_slice() != expected.as_slice() {
        return Err(StyleError::IntegrityMismatch {
            expected: hex::encode(&expected),
            actual: hex::encode(actual),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_integrity(data: &[u8]) -> String {
        let hash = Sha512::digest(data);
        format!("sha512-{}", STANDARD.encode(hash))
    }

    #[test]
    fn valid_integrity_passes() {
        let data = b"hello world";
        let integrity = make_integrity(data);
        assert!(verify_sha512(&integrity, data).is_ok());
    }

    #[test]
    fn tampered_data_fails() {
        let data = b"hello world";
        let integrity = make_integrity(data);
        assert!(verify_sha512(&integrity, b"tampered").is_err());
    }

    #[test]
    fn multi_hash_integrity_picks_sha512() {
        let data = b"hello world";
        let sha512 = make_integrity(data);
        let combined = format!("sha256-AAAA {}", sha512);
        assert!(verify_sha512(&combined, data).is_ok());
    }

    #[test]
    fn unsupported_algorithm_is_rejected() {
        let result = verify_sha512("sha1-abc123", b"data");
        assert!(matches!(result, Err(StyleError::UnsupportedIntegrity(_))));
    }
}
