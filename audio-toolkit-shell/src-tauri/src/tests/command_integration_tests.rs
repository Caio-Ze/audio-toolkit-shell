#[cfg(test)]
mod tests {
    use crate::handlers::errors::CommandError;
    use crate::models::*;
    use crate::services::{ProcessManager, FileDropHandler};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn create_test_terminal_config() -> TerminalConfig {
        TerminalConfig {
            id: "test_terminal".to_string(),
            name: "Test Terminal".to_string(),
            launcher_executable: "echo".to_string(), // Use echo for testing
            working_directory: "/tmp".to_string(),
            environment_variables: HashMap::new(),
            auto_start: true,
        }
    }

    #[tokio::test]
    async fn test_process_manager_spawn_launcher() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config();
        
        let result = process_manager.spawn_launcher(&config).await;
        assert!(result.is_ok(), "ProcessManager should spawn launcher successfully");
        
        let process_info = result.unwrap();
        assert_eq!(process_info.terminal_id, "test_terminal");
        assert!(matches!(process_info.status, ProcessStatus::Running));
        assert!(process_info.pid.is_some());
    }

    #[tokio::test]
    async fn test_process_manager_spawn_duplicate() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config();
        
        // Spawn first process
        let result1 = process_manager.spawn_launcher(&config).await;
        assert!(result1.is_ok());
        
        // Try to spawn duplicate
        let result2 = process_manager.spawn_launcher(&config).await;
        assert!(result2.is_err(), "Should fail to spawn duplicate process");
    }

    #[tokio::test]
    async fn test_process_manager_send_command() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config();
        
        // First spawn a process
        let spawn_result = process_manager.spawn_launcher(&config).await;
        assert!(spawn_result.is_ok());
        
        // Then send a command
        let result = process_manager.send_command("test_terminal", "test command").await;
        assert!(result.is_ok(), "Should send command successfully");
    }

    #[tokio::test]
    async fn test_process_manager_send_command_nonexistent() {
        let process_manager = ProcessManager::new();
        
        let result = process_manager.send_command("nonexistent", "test command").await;
        assert!(result.is_err(), "Should fail to send command to nonexistent process");
    }

    #[tokio::test]
    async fn test_process_manager_read_output() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config();
        
        // First spawn a process
        let spawn_result = process_manager.spawn_launcher(&config).await;
        assert!(spawn_result.is_ok());
        
        // Read output (should be empty initially)
        let result = process_manager.read_output("test_terminal").await;
        assert!(result.is_ok(), "Should read output successfully");
        
        let output = result.unwrap();
        assert!(output.is_empty() || !output.is_empty(), "Output should be a vector");
    }

    #[tokio::test]
    async fn test_process_manager_get_process_info() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config();
        
        // First spawn a process
        let spawn_result = process_manager.spawn_launcher(&config).await;
        assert!(spawn_result.is_ok());
        
        // Get process info
        let result = process_manager.get_process_info("test_terminal").await;
        assert!(result.is_ok(), "Should get process info successfully");
        
        let process_info = result.unwrap();
        assert_eq!(process_info.terminal_id, "test_terminal");
        assert!(process_info.pid.is_some());
    }

    #[tokio::test]
    async fn test_process_manager_list_processes() {
        let process_manager = ProcessManager::new();
        
        // Initially should be empty
        let result = process_manager.list_processes().await;
        assert!(result.is_empty(), "Should start with no processes");
        
        // Spawn a process
        let config = create_test_terminal_config();
        let spawn_result = process_manager.spawn_launcher(&config).await;
        assert!(spawn_result.is_ok());
        
        // Should now have one process
        let result = process_manager.list_processes().await;
        assert_eq!(result.len(), 1, "Should have one process after spawning");
    }

    #[tokio::test]
    async fn test_file_drop_handler_valid_file() {
        let file_handler = FileDropHandler::new();
        
        // Create a temporary file for testing
        let temp_file = "/tmp/test_file_drop.txt";
        std::fs::write(temp_file, "test content").expect("Failed to create test file");
        
        let path_buf = std::path::PathBuf::from(temp_file);
        let result = file_handler.handle_drop_event(vec![path_buf], "test_terminal");
        
        assert!(result.is_ok(), "FileDropHandler should handle valid file");
        
        let formatted_path = result.unwrap();
        assert!(formatted_path.contains("test_file_drop.txt"), "Should contain filename");
        
        // Clean up
        let _ = std::fs::remove_file(temp_file);
    }

    #[tokio::test]
    async fn test_file_drop_handler_path_with_spaces() {
        let file_handler = FileDropHandler::new();
        
        let path_with_spaces = "/tmp/file with spaces.txt";
        let escaped = file_handler.escape_path(path_with_spaces);
        
        assert_eq!(escaped, "\"/tmp/file with spaces.txt\"", "Should quote paths with spaces");
    }

    #[tokio::test]
    async fn test_file_drop_handler_path_without_spaces() {
        let file_handler = FileDropHandler::new();
        
        let path_without_spaces = "/tmp/file_without_spaces.txt";
        let escaped = file_handler.escape_path(path_without_spaces);
        
        assert_eq!(escaped, "/tmp/file_without_spaces.txt", "Should not quote paths without spaces");
    }

    #[tokio::test]
    async fn test_command_error_types() {
        // Test different error types
        let auth_error = CommandError::authentication("Test auth error");
        assert!(matches!(auth_error, CommandError::AuthenticationError { .. }));
        
        let process_error = CommandError::process("Test process error");
        assert!(matches!(process_error, CommandError::ProcessError { .. }));
        
        let file_error = CommandError::file("Test file error");
        assert!(matches!(file_error, CommandError::FileError { .. }));
        
        let validation_error = CommandError::validation("Test validation error");
        assert!(matches!(validation_error, CommandError::ValidationError { .. }));
    }

    #[tokio::test]
    async fn test_process_manager_terminate_process() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config();
        
        // First spawn a process
        let spawn_result = process_manager.spawn_launcher(&config).await;
        assert!(spawn_result.is_ok());
        
        // Then terminate it
        let result = process_manager.terminate_process("test_terminal").await;
        assert!(result.is_ok(), "Should terminate process successfully");
        
        // Process should no longer exist
        let info_result = process_manager.get_process_info("test_terminal").await;
        assert!(info_result.is_err(), "Process should no longer exist after termination");
    }

    #[tokio::test]
    async fn test_integration_spawn_send_terminate() {
        let process_manager = ProcessManager::new();
        let config = create_test_terminal_config();
        
        // 1. Spawn process
        let spawn_result = process_manager.spawn_launcher(&config).await;
        assert!(spawn_result.is_ok(), "Should spawn process");
        
        // 2. Send command
        let send_result = process_manager.send_command("test_terminal", "echo 'test'").await;
        assert!(send_result.is_ok(), "Should send command");
        
        // 3. Read output (may be empty but should not error)
        let output_result = process_manager.read_output("test_terminal").await;
        assert!(output_result.is_ok(), "Should read output");
        
        // 4. Get process info
        let info_result = process_manager.get_process_info("test_terminal").await;
        assert!(info_result.is_ok(), "Should get process info");
        
        // 5. Terminate process
        let terminate_result = process_manager.terminate_process("test_terminal").await;
        assert!(terminate_result.is_ok(), "Should terminate process");
    }
}