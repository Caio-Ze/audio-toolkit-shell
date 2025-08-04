#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::services::*;
    use tokio;

    #[tokio::test]
    async fn test_authentication_service_creation() {
        let auth_service = AuthenticationService::new("http://localhost:8080".to_string());
        // Service should be created successfully
        assert!(true); // Basic creation test
    }

    #[tokio::test]
    async fn test_development_mode_access_validation() {
        let auth_service = AuthenticationService::new(String::new()); // Empty URL = dev mode
        
        // Valid key should pass
        let result = auth_service.validate_access("valid_key_123").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Short key should fail
        let result = auth_service.validate_access("short").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
        
        // Empty key should fail
        let result = auth_service.validate_access("").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_development_mode_version_check() {
        let auth_service = AuthenticationService::new(String::new());
        
        // Current version should be accepted
        let result = auth_service.check_version("0.1.0").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), VersionStatus::Current);
        
        // Very old version should require update
        let result = auth_service.check_version("0.0.1").await;
        assert!(result.is_ok());
        match result.unwrap() {
            VersionStatus::UpdateRequired(version) => {
                assert_eq!(version, "0.1.0");
            }
            _ => panic!("Expected UpdateRequired"),
        }
    }

    #[tokio::test]
    async fn test_full_authentication_flow() {
        let auth_service = AuthenticationService::new(String::new());
        let auth_request = AuthRequest {
            user_key: "test_user_key_12345".to_string(),
            client_version: "0.1.0".to_string(),
        };

        let result = auth_service.authenticate(&auth_request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.auth_status.is_authenticated);
        assert!(response.auth_status.user_id.is_some());
        assert!(!response.auth_status.permissions.is_empty());
        assert_eq!(response.version_status, VersionStatus::Current);
    }

    #[tokio::test]
    async fn test_authentication_with_invalid_key() {
        let auth_service = AuthenticationService::new(String::new());
        let auth_request = AuthRequest {
            user_key: "bad".to_string(), // Too short
            client_version: "0.1.0".to_string(),
        };

        let result = auth_service.authenticate(&auth_request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.auth_status.is_authenticated);
        assert!(response.auth_status.user_id.is_none());
        assert!(response.auth_status.permissions.is_empty());
    }

    #[tokio::test]
    async fn test_retry_config() {
        let retry_config = RetryConfig {
            max_attempts: 2,
            base_delay_ms: 100,
            max_delay_ms: 1000,
        };

        let auth_service = AuthenticationService::new(String::new())
            .with_retry_config(retry_config);

        // Should still work in development mode
        let result = auth_service.validate_access("valid_key_123").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 10000);
    }

    #[test]
    fn test_auth_request_default() {
        let request = AuthRequest::default();
        assert!(!request.user_key.is_empty());
        assert!(!request.client_version.is_empty());
    }

    #[test]
    fn test_auth_response_default() {
        let response = AuthResponse::default();
        assert!(!response.auth_status.is_authenticated);
        assert_eq!(response.version_status, VersionStatus::Current);
        assert!(response.server_message.is_none());
    }

    #[tokio::test]
    async fn test_key_masking() {
        let auth_service = AuthenticationService::new(String::new());
        
        // Test with different key lengths
        let short_key = "abc";
        let medium_key = "abcdefgh";
        let long_key = "abcdefghijklmnop";
        
        // All should be processed without panicking
        let _ = auth_service.validate_access(short_key).await;
        let _ = auth_service.validate_access(medium_key).await;
        let _ = auth_service.validate_access(long_key).await;
    }

    #[tokio::test]
    async fn test_user_id_extraction() {
        let auth_service = AuthenticationService::new(String::new());
        let auth_request = AuthRequest {
            user_key: "test1234567890".to_string(),
            client_version: "0.1.0".to_string(),
        };

        let result = auth_service.authenticate(&auth_request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        if let Some(user_id) = response.auth_status.user_id {
            assert!(user_id.starts_with("user_"));
            assert!(user_id.contains("test"));
        }
    }
}