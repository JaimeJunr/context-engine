// Cache de operações custosas (RD-22)
//
// Armazena resultados de: variantes de query, julgamentos de pertinência.
// A chave é o SHA256 da entrada serializada.

use anyhow::Result;
use chrono::Utc;
use hex;
use sha2::{Digest, Sha256};

use super::store;

pub const OP_QUERY_VARIANTS: &str = "query_variants";
pub const OP_RERANK_JUDGMENT: &str = "rerank_judgment";

fn hash_input(input: &str) -> String {
    let mut h = Sha256::new();
    h.update(input.as_bytes());
    hex::encode(h.finalize())
}

pub fn get_cached<T: serde::de::DeserializeOwned>(input: &str, op_type: &str) -> Option<T> {
    let key = hash_input(input);
    store::cache_get_op(&key, op_type)
        .ok()
        .flatten()
        .and_then(|json| serde_json::from_str(&json).ok())
}

pub fn set_cached<T: serde::Serialize>(input: &str, op_type: &str, value: &T) -> Result<()> {
    let key = hash_input(input);
    let json = serde_json::to_string(value)?;
    let now = Utc::now().to_rfc3339();
    store::cache_set_op(&key, op_type, &json, &now)
}
