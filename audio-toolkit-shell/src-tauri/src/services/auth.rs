use crate::models::{AuthRequest, AuthResponse, AuthStatus, VersionStatus};
use anyhow::{anyhow, Result};

use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 10000,
        }
    }
}

pub struct AuthenticationService {
    client: Client,
    server_url: String,
    retry_config: RetryConfig,
}

impl AuthenticationService {
    pub fn new(server_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("AudioToolkitShell/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            server_url,
            retry_config: RetryConfig::default(),
        }
    }

    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    pub async fn authenticate(&self, request: &AuthRequest) -> Result<AuthResponse> {
        log::info!("Authenticating user with version: {}", request.client_version);

        let auth_result = self.validate_access(&request.user_key).await?;
        let version_result = self.check_version(&request.client_version).await?;

        let auth_status = if auth_result {
            AuthStatus {
                is_authenticated: true,
                user_id: Some(self.extract_user_id(&request.user_key)),
                expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(24)),
                permissions: vec![
                    "terminal_access".to_string(),
                    "file_drop".to_string(),
                    "process_management".to_string(),
                ],
            }
        } else {
            AuthStatus::default()
        };

        Ok(AuthResponse {
            auth_status,
            version_status: version_result,
            server_message: None,
        })
    }

    pub async fn validate_access(&self, user_key: &str) -> Result<bool> {
        log::info!("Validating access for user key: {}", self.mask_key(user_key));

        // For development, we'll simulate server communication
        // In production, this would make an actual HTTP request
        if self.is_development_mode() {
            return self.simulate_access_validation(user_key).await;
        }

        let url = format!("{}/api/auth/validate", self.server_url);
        let payload = json!({
            "user_key": user_key,
            "timestamp": chrono::Utc::now().timestamp()
        });

        let response = self
            .retry_request(|| async {
                self.client
                    .post(&url)
                    .json(&payload)
                    .send()
                    .await?
                    .error_for_status()
            })
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["valid"].as_bool().unwrap_or(false))
    }

    pub async fn check_version(&self, current_version: &str) -> Result<VersionStatus> {
        log::info!("Checking version: {}", current_version);

        // For development, simulate version checking
        if self.is_development_mode() {
            return self.simulate_version_check(current_version).await;
        }

        let url = format!("{}/api/version/check", self.server_url);
        let payload = json!({
            "current_version": current_version,
            "platform": "macos"
        });

        let response = self
            .retry_request(|| async {
                self.client
                    .post(&url)
                    .json(&payload)
                    .send()
                    .await?
                    .error_for_status()
            })
            .await?;

        let result: serde_json::Value = response.json().await?;
        
        match result["status"].as_str() {
            Some("current") => Ok(VersionStatus::Current),
            Some("update_required") => {
                let required_version = result["required_version"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                Ok(VersionStatus::UpdateRequired(required_version))
            }
            Some("update_available") => {
                let available_version = result["available_version"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                Ok(VersionStatus::UpdateAvailable(available_version))
            }
            _ => Err(anyhow!("Invalid version status response")),
        }
    }

    async fn retry_request<F, Fut>(&self, request_fn: F) -> Result<reqwest::Response>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
    {
        let mut last_error = None;

        for attempt in 1..=self.retry_config.max_attempts {
            match request_fn().await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < self.retry_config.max_attempts {
                        let delay = self.calculate_delay(attempt);
                        log::warn!(
                            "Request failed (attempt {}/{}), retrying in {}ms: {}",
                            attempt,
                            self.retry_config.max_attempts,
                            delay,
                            last_error.as_ref().unwrap()
                        );
                        sleep(Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        Err(anyhow!(
            "Request failed after {} attempts: {}",
            self.retry_config.max_attempts,
            last_error.unwrap()
        ))
    }

    fn calculate_delay(&self, attempt: u32) -> u64 {
        let delay = self.retry_config.base_delay_ms * (2_u64.pow(attempt - 1));
        delay.min(self.retry_config.max_delay_ms)
    }

    fn is_development_mode(&self) -> bool {
        // Check if we're in development mode (no real server configured)
        self.server_url.is_empty() || self.server_url == "http://localhost" || cfg!(debug_assertions)
    }

    async fn simulate_access_validation(&self, user_key: &str) -> Result<bool> {
        // Simulate network delay
        sleep(Duration::from_millis(100)).await;
        
        // For development, accept any non-empty key
        Ok(!user_key.is_empty() && user_key.len() >= 8)
    }

    async fn simulate_version_check(&self, current_version: &str) -> Result<VersionStatus> {
        // Simulate network delay
        sleep(Duration::from_millis(150)).await;
        
        // For development, always return current unless version is very old
        if current_version.starts_with("0.0") {
            Ok(VersionStatus::UpdateRequired("0.1.0".to_string()))
        } else {
            Ok(VersionStatus::Current)
        }
    }

    fn extract_user_id(&self, user_key: &str) -> String {
        // In development, create a user ID from the key
        // In production, this would come from the server response
        format!("user_{}", &user_key[..4.min(user_key.len())])
    }

    fn mask_key(&self, key: &str) -> String {
        if key.len() <= 8 {
            "*".repeat(key.len())
        } else {
            format!("{}***{}", &key[..2], &key[key.len()-2..])
        }
    }
}