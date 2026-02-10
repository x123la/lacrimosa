//! # Access Control & Audit
//!
//! API key management, scope-based authorization, and audit logging.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use tokio::sync::RwLock;

/// Permission scopes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Read,
    Write,
    Admin,
}

/// An API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub label: String,
    /// The actual key value (only shown once at creation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// SHA-256 hash of the key (stored for comparison).
    pub key_hash: String,
    pub scopes: Vec<Scope>,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub revoked: bool,
}

/// Audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub detail: String,
    pub ip: Option<String>,
}

/// Request to create an API key.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiKeyRequest {
    pub label: String,
    pub scopes: Vec<Scope>,
}

/// The auth layer state.
pub struct AuthLayer {
    pub api_keys: RwLock<Vec<ApiKey>>,
    pub audit_log: RwLock<VecDeque<AuditEntry>>,
    audit_capacity: usize,
}

impl AuthLayer {
    pub fn new(audit_capacity: usize) -> Self {
        Self {
            api_keys: RwLock::new(Vec::new()),
            audit_log: RwLock::new(VecDeque::with_capacity(audit_capacity)),
            audit_capacity,
        }
    }

    /// Create a new API key. Returns the key with the raw value (shown once).
    pub async fn create_key(&self, req: CreateApiKeyRequest) -> ApiKey {
        let raw_key = format!("cz_{}", uuid::Uuid::new_v4().as_simple());
        let key_hash = sha256_hex(&raw_key);

        let api_key = ApiKey {
            id: format!("key-{}", uuid::Uuid::new_v4().as_simple()),
            label: req.label,
            key: Some(raw_key.clone()),
            key_hash,
            scopes: req.scopes,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used_at: None,
            revoked: false,
        };

        let mut keys = self.api_keys.write().await;
        keys.push(api_key.clone());

        self.log_audit(
            "system".into(),
            "create_key".into(),
            format!("api_key:{}", api_key.id),
            format!("Created API key '{}'", api_key.label),
            None,
        )
        .await;

        api_key
    }

    /// Revoke an API key.
    pub async fn revoke_key(&self, key_id: &str) -> Result<(), String> {
        let mut keys = self.api_keys.write().await;
        let key = keys
            .iter_mut()
            .find(|k| k.id == key_id)
            .ok_or_else(|| format!("Key '{}' not found", key_id))?;
        key.revoked = true;
        Ok(())
    }

    /// List all API keys (without raw values).
    pub async fn list_keys(&self) -> Vec<ApiKey> {
        let keys = self.api_keys.read().await;
        keys.iter()
            .map(|k| {
                let mut k = k.clone();
                k.key = None; // Never expose raw key after creation
                k
            })
            .collect()
    }

    /// Validate a bearer token. Returns the API key if valid.
    pub async fn validate_token(&self, token: &str) -> Option<ApiKey> {
        let hash = sha256_hex(token);
        let mut keys = self.api_keys.write().await;
        let key = keys
            .iter_mut()
            .find(|k| constant_time_eq(&k.key_hash, &hash) && !k.revoked)?;
        key.last_used_at = Some(chrono::Utc::now().to_rfc3339());
        let mut result = key.clone();
        result.key = None;
        Some(result)
    }

    pub fn has_scope(&self, key: &ApiKey, required: Scope) -> bool {
        if key.scopes.contains(&Scope::Admin) {
            return true;
        }
        key.scopes.contains(&required)
    }

    /// Log an audit entry.
    pub async fn log_audit(
        &self,
        actor: String,
        action: String,
        resource: String,
        detail: String,
        ip: Option<String>,
    ) {
        let entry = AuditEntry {
            id: format!("audit-{}", uuid::Uuid::new_v4().as_simple()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            actor,
            action,
            resource,
            detail,
            ip,
        };

        let mut log = self.audit_log.write().await;
        if log.len() >= self.audit_capacity {
            log.pop_front();
        }
        log.push_back(entry);
    }

    /// Get recent audit log entries.
    pub async fn get_audit_log(&self, limit: usize) -> Vec<AuditEntry> {
        let log = self.audit_log.read().await;
        log.iter().rev().take(limit).cloned().collect()
    }
}

fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    format!("{:x}", digest)
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}
