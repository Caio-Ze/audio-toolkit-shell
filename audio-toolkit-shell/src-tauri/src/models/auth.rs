use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub user_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VersionStatus {
    Current,
    UpdateRequired(String),
    UpdateAvailable(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub user_key: String,
    pub client_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub auth_status: AuthStatus,
    pub version_status: VersionStatus,
    pub server_message: Option<String>,
}

impl Default for AuthStatus {
    fn default() -> Self {
        Self {
            is_authenticated: false,
            user_id: None,
            expires_at: None,
            permissions: vec![],
        }
    }
}

impl Default for AuthRequest {
    fn default() -> Self {
        Self {
            user_key: "default_dev_key".to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl Default for AuthResponse {
    fn default() -> Self {
        Self {
            auth_status: AuthStatus::default(),
            version_status: VersionStatus::Current,
            server_message: None,
        }
    }
}