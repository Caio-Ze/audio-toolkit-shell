#[cfg(test)]
mod tests {
    use crate::models::*;
    use std::collections::HashMap;
    use chrono::Utc;

    #[test]
    fn test_terminal_config_serialization() {
        let mut env_vars = HashMap::new();
        env_vars.insert("PATH".to_string(), "/usr/bin".to_string());
        
        let config = TerminalConfig {
            id: "test_terminal".to_string(),
            name: "Test Terminal".to_string(),
            launcher_executable: "/path/to/launcher".to_string(),
            working_directory: "/tmp".to_string(),
            environment_variables: env_vars,
            auto_start: true,
        };

        // Test serialization
        let json = serde_json::to_string(&config).expect("Failed to serialize");
        assert!(json.contains("test_terminal"));
        assert!(json.contains("Test Terminal"));

        // Test deserialization
        let deserialized: TerminalConfig = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_auth_status_serialization() {
        let auth_status = AuthStatus {
            is_authenticated: true,
            user_id: Some("test_user".to_string()),
            expires_at: Some(Utc::now()),
            permissions: vec!["read".to_string(), "write".to_string()],
        };

        // Test serialization
        let json = serde_json::to_string(&auth_status).expect("Failed to serialize");
        assert!(json.contains("test_user"));
        assert!(json.contains("true"));

        // Test deserialization
        let deserialized: AuthStatus = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(auth_status, deserialized);
    }

    #[test]
    fn test_process_status_serialization() {
        let statuses = vec![
            ProcessStatus::Starting,
            ProcessStatus::Running,
            ProcessStatus::Idle,
            ProcessStatus::Processing,
            ProcessStatus::Error("Test error".to_string()),
            ProcessStatus::Terminated,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).expect("Failed to serialize");
            let deserialized: ProcessStatus = serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_version_status_serialization() {
        let statuses = vec![
            VersionStatus::Current,
            VersionStatus::UpdateRequired("1.2.0".to_string()),
            VersionStatus::UpdateAvailable("1.1.5".to_string()),
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).expect("Failed to serialize");
            let deserialized: VersionStatus = serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_terminal_config_default() {
        let config = TerminalConfig::default();
        assert_eq!(config.id, "");
        assert_eq!(config.name, "");
        assert_eq!(config.working_directory, ".");
        assert!(config.environment_variables.is_empty());
        assert!(config.auto_start);
    }

    #[test]
    fn test_auth_status_default() {
        let auth = AuthStatus::default();
        assert!(!auth.is_authenticated);
        assert!(auth.user_id.is_none());
        assert!(auth.expires_at.is_none());
        assert!(auth.permissions.is_empty());
    }

    #[test]
    fn test_process_info_creation() {
        let process_info = ProcessInfo {
            terminal_id: "terminal_1".to_string(),
            status: ProcessStatus::Running,
            pid: Some(1234),
            started_at: Some(Utc::now()),
            last_activity: Some(Utc::now()),
            cpu_usage: Some(25.5),
            memory_usage: Some(1024 * 1024), // 1MB
        };

        assert_eq!(process_info.terminal_id, "terminal_1");
        assert_eq!(process_info.status, ProcessStatus::Running);
        assert_eq!(process_info.pid, Some(1234));
        assert!(process_info.cpu_usage.is_some());
        assert!(process_info.memory_usage.is_some());
    }
}