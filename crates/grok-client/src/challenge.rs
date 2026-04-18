use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64;
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::error::{GrokError, Result};

const EPOCH: u64 = 1682924400;

const HEADER_LEN: usize = 49;
const TOKEN_LEN: usize = HEADER_LEN + 4 + 16 + 1;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ChallengeConfig {
    static_header: [u8; HEADER_LEN],
    static_suffix: String,
    trailer_byte: u8,
}

impl ChallengeConfig {
    pub fn new(static_header_hex: &str, static_suffix: &str, trailer_byte: u8) -> Result<Self> {
        let decoded = hex_decode(static_header_hex)
            .map_err(|e| GrokError::Config(format!("Invalid static_header hex: {e}")))?;

        let static_header: [u8; HEADER_LEN] = decoded.try_into().map_err(|v: Vec<u8>| {
            GrokError::Config(format!(
                "static_header must be {HEADER_LEN} bytes, got {}",
                v.len()
            ))
        })?;

        Ok(Self {
            static_header,
            static_suffix: static_suffix.to_owned(),
            trailer_byte,
        })
    }

    pub fn generate_token(&self, path: &str, method: &str) -> String {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(EPOCH);
        let counter = now_secs.saturating_sub(EPOCH);
        let counter_str = counter.to_string();

        let hash_input = format!("{method}!{path}!{counter_str}{}", self.static_suffix);
        let hash = Sha256::digest(hash_input.as_bytes());

        let mut raw = [0u8; TOKEN_LEN];
        raw[..HEADER_LEN].copy_from_slice(&self.static_header);
        raw[HEADER_LEN..HEADER_LEN + 4].copy_from_slice(&(counter as u32).to_le_bytes());
        raw[HEADER_LEN + 4..HEADER_LEN + 20].copy_from_slice(&hash[..16]);
        raw[TOKEN_LEN - 1] = self.trailer_byte;

        let xor_key: u8 = rand::rng().random();
        for byte in &mut raw {
            *byte ^= xor_key;
        }

        BASE64.encode(raw)
    }

    pub fn generate_headers(&self, path: &str, method: &str) -> (String, String) {
        let token = self.generate_token(path, method);
        let request_id = uuid::Uuid::new_v4().to_string();
        (token, request_id)
    }
}

impl crate::client::TokenProvider for ChallengeConfig {
    fn generate(
        &self,
        path: &str,
        method: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<crate::client::TokenPair>> + Send + '_>,
    > {
        let (statsig_id, request_id) = self.generate_headers(path, method);
        Box::pin(async move {
            Ok(crate::client::TokenPair {
                statsig_id,
                request_id,
            })
        })
    }
}

fn hex_decode(hex: &str) -> std::result::Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Odd-length hex string".into());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_hex() -> String {
        "00".repeat(HEADER_LEN)
    }

    #[test]
    fn rejects_wrong_length_header() {
        assert!(ChallengeConfig::new("00", "suffix", 3).is_err());
    }

    #[test]
    fn rejects_odd_hex() {
        assert!(ChallengeConfig::new("0", "suffix", 3).is_err());
    }

    #[test]
    fn rejects_invalid_hex_chars() {
        assert!(ChallengeConfig::new(&"zz".repeat(HEADER_LEN), "suffix", 3).is_err());
    }

    #[test]
    fn token_is_valid_base64_and_correct_length() {
        let cfg = ChallengeConfig::new(&sample_hex(), "suffix", 3).unwrap();
        let token = cfg.generate_token("/x", "POST");
        let decoded = BASE64.decode(token.as_bytes()).expect("valid base64");
        assert_eq!(decoded.len(), TOKEN_LEN);
    }

    #[test]
    fn tokens_differ_across_calls_due_to_xor() {
        let cfg = ChallengeConfig::new(&sample_hex(), "suffix", 3).unwrap();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..32 {
            seen.insert(cfg.generate_token("/x", "POST"));
        }
        assert!(seen.len() > 1, "XOR randomization should vary token");
    }

    #[test]
    fn request_id_is_uuid() {
        let cfg = ChallengeConfig::new(&sample_hex(), "suffix", 3).unwrap();
        let (_, req_id) = cfg.generate_headers("/x", "POST");
        assert!(uuid::Uuid::parse_str(&req_id).is_ok());
    }
}
