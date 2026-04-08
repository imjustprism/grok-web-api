use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64;
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::error::{GrokError, Result};

const EPOCH: u64 = 1682924400;

const HEADER_LEN: usize = 49;
const TOKEN_LEN: usize = HEADER_LEN + 4 + 16 + 1;

#[derive(Debug, Clone)]
pub struct ChallengeConfig {
    static_header: [u8; HEADER_LEN],
    static_suffix: String,
    trailer_byte: u8,
}

impl ChallengeConfig {
    pub fn new(
        static_header_hex: &str,
        static_suffix: &str,
        trailer_byte: u8,
    ) -> Result<Self> {
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
            .unwrap_or_default()
            .as_secs();
        let counter = now_secs - EPOCH;
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
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<crate::client::TokenPair>> + Send + '_>> {
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
