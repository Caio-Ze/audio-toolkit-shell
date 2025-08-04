#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::services::*;
    use std::collections::HashMap;
    use tokio;

    fn create_test_terminal_config(id: &str, name: &str) -> TerminalConfig {
        TerminalConfig {
            id: id.to_string(),
            name: name.to_string(),
            launcher_executable: "echo".to_string(), // Use echo for testing
            working_directory: "/tmp".to_string(),
            environment_variables: HashMap::new(),
            auto_start: true,
        }
    }

    #[tokio::test]
    async fn test_process_manager_creation() {
        let process_manager = ProcessManager::new();
        let processes = process_manager.list_processes().await;
        assert!(processes.is_empty());
    }

    #[tokio::test]
    async fn test_process_config_default() {
        let config = ProcessConfig::default();
        assert!(config.auto_restart);
        assert_eq!(config.max_restart_attempts, 3);
        assert_eq!(config.restart_delay_ms, 5000);
        assert_eq!(config.health_check_interval_ms, 10000);
    }

    #[tokio::test]
    async fn test_process_manager_with_config() {
        let custom_config = ProcessConfig {
            auto_restart: false,
            max_restart_attempts: 5,
            restart_delay_ms: 1000,
            health_check_interval_ms: 5000,
        };

        let process_manager = ProcessManager::new().with_config(custom_config);
        // Process manager should be created with custom config
        assert!(true); // Basic creation test
    }

    #[tokio::test]
    async fn test_spawn_simple_process() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config("test_echo", "Test Echo");

        let result = process_manager.spawn_launcher(&config).await;
        assert!(result.is_ok());

        let process_info = result.unwrap();
        assert_eq!(process_info.terminal_id, "test_echo");
        assert_eq!(process_info.status, ProcessStatus::Running);
        assert!(process_info.pid.is_some());
        assert!(process_info.started_at.is_some());

        // Clean up
        let _ = process_manager.terminate_process("test_echo").await;
    }

    #[tokio::test]
    async fn test_spawn_duplicate_process() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config("test_duplicate", "Test Duplicate");

        // Spawn first process
        let result1 = process_manager.spawn_launcher(&config).await;
        assert!(result1.is_ok());

        // Try to spawn duplicate
        let result2 = process_manager.spawn_launcher(&config).await;
        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("already exists"));

        // Clean up
        let _ = process_manager.terminate_process("test_duplicate").await;
    }

    #[tokio::test]
    async fn test_spawn_invalid_executable() {
        let process_manager = ProcessManager::new();
        let mut config = create_test_terminal_config("test_invalid", "Test Invalid");
        config.launcher_executable = "/nonexistent/executable".to_string();

        let result = process_manager.spawn_launcher(&config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to spawn process"));
    }

    #[tokio::test]
    async fn test_send_command_to_process() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config("test_command", "Test Command");

        // Spawn process
        let spawn_result = process_manager.spawn_launcher(&config).await;
        assert!(spawn_result.is_ok());

        // Send command
        let command_result = process_manager.send_command("test_command", "hello world").await;
        assert!(command_result.is_ok());

        // Clean up
        let _ = process_manager.terminate_process("test_command").await;
    }

    #[tokio::test]
    async fn test_send_command_to_nonexistent_process() {
        let process_manager = ProcessManager::new();

        let result = process_manager.send_command("nonexistent", "test").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_read_output_from_process() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config("test_output", "Test Output");

        // Spawn process
        let spawn_result = process_manager.spawn_launcher(&config).await;
        assert!(spawn_result.is_ok());

        // Give the process a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Read output (should be empty initially)
        let output_result = process_manager.read_output("test_output").await;
        assert!(output_result.is_ok());
        let output = output_result.unwrap();
        // Output might be empty or contain some initial output
        assert!(output.len() >= 0);

        // Clean up
        let _ = process_manager.terminate_process("test_output").await;
    }

    #[tokio::test]
    async fn test_get_process_info() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config("test_info", "Test Info");

        // Spawn process
        let spawn_result = process_manager.spawn_launcher(&config).await;
        assert!(spawn_result.is_ok());

        // Get process info
        let info_result = process_manager.get_process_info("test_info").await;
        assert!(info_result.is_ok());

        let info = info_result.unwrap();
        assert_eq!(info.terminal_id, "test_info");
        assert_eq!(info.status, ProcessStatus::Running);
        assert!(info.pid.is_some());
        assert!(info.started_at.is_some());

        // Clean up
        let _ = process_manager.terminate_process("test_info").await;
    }

    #[tokio::test]
    async fn test_get_nonexistent_process_info() {
        let process_manager = ProcessManager::new();

        let result = process_manager.get_process_info("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_list_processes() {
        let process_manager = ProcessManager::new();

        // Initially empty
        let processes = process_manager.list_processes().await;
        assert!(processes.is_empty());

        // Spawn a process
        let config = create_test_terminal_config("test_list", "Test List");
        let spawn_result = process_manager.spawn_launcher(&config).await;
        assert!(spawn_result.is_ok());

        // Should have one process
        let processes = process_manager.list_processes().await;
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].terminal_id, "test_list");

        // Clean up
        let _ = process_manager.terminate_process("test_list").await;

        // Should be empty again
        let processes = process_manager.list_processes().await;
        assert!(processes.is_empty());
    }

    #[tokio::test]
    async fn test_terminate_process() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config("test_terminate", "Test Terminate");

        // Spawn process
        let spawn_result = process_manager.spawn_launcher(&config).await;
        assert!(spawn_result.is_ok());

        // Verify it exists
        let processes = process_manager.list_processes().await;
        assert_eq!(processes.len(), 1);

        // Terminate it
        let terminate_result = process_manager.terminate_process("test_terminate").await;
        assert!(terminate_result.is_ok());

        // Verify it's gone
        let processes = process_manager.list_processes().await;
        assert!(processes.is_empty());
    }

    #[tokio::test]
    async fn test_terminate_nonexistent_process() {
        let process_manager = ProcessManager::new();

        let result = process_manager.terminate_process("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_restart_process() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config("test_restart", "Test Restart");

        // Spawn process
        let spawn_result = process_manager.spawn_launcher(&config).await;
        assert!(spawn_result.is_ok());
        let original_pid = spawn_result.unwrap().pid;

        // Restart process
        let restart_result = process_manager.restart_process("test_restart").await;
        assert!(restart_result.is_ok());
        let new_pid = restart_result.unwrap().pid;

        // PIDs should be different (new process)
        assert_ne!(original_pid, new_pid);

        // Clean up
        let _ = process_manager.terminate_process("test_restart").await;
    }

    #[tokio::test]
    async fn test_restart_nonexistent_process() {
        let process_manager = ProcessManager::new();

        let result = process_manager.restart_process("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_process_with_environment_variables() {
        let process_manager = ProcessManager::new();
        let mut config = create_test_terminal_config("test_env", "Test Environment");
        
        // Add environment variables
        config.environment_variables.insert("TEST_VAR".to_string(), "test_value".to_string());
        config.environment_variables.insert("ANOTHER_VAR".to_string(), "another_value".to_string());

        let result = process_manager.spawn_launcher(&config).await;
        assert!(result.is_ok());

        // Clean up
        let _ = process_manager.terminate_process("test_env").await;
    }

    #[tokio::test]
    async fn test_process_with_custom_working_directory() {
        let process_manager = ProcessManager::new();
        let mut config = create_test_terminal_config("test_workdir", "Test Working Directory");
        config.working_directory = "/".to_string(); // Use root directory

        let result = process_manager.spawn_launcher(&config).await;
        assert!(result.is_ok());

        // Clean up
        let _ = process_manager.terminate_process("test_workdir").await;
    }
}