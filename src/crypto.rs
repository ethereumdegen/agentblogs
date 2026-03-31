use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// SHA-256 hash a string, returning hex-encoded digest.
pub fn sha256_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a new API key with `ab_live_` prefix.
pub fn generate_api_key() -> String {
    let mut random_bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut random_bytes);
    format!("ab_live_{}", URL_SAFE_NO_PAD.encode(&random_bytes))
}
