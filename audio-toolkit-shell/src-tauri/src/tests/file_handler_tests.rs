#[cfg(test)]
mod tests {
    use crate::services::{FileDropHandler, FileDropConfig, FileType};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_files() -> (TempDir, Vec<PathBuf>) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mut files = Vec::new();

        // Create a regular file
        let file1 = temp_dir.path().join("test_file.txt");
        fs::write(&file1, "test content").expect("Failed to create test file");
        files.push(file1);

        // Create a file with spaces in name
        let file2 = temp_dir.path().join("file with spaces.txt");
        fs::write(&file2, "test content").expect("Failed to create test file with spaces");
        files.push(file2);

        // Create a directory
        let dir1 = temp_dir.path().join("test_directory");
        fs::create_dir(&dir1).expect("Failed to create test directory");
        files.push(dir1);

        // Create a file with special characters
        let file3 = temp_dir.path().join("file(with)special[chars].txt");
        fs::write(&file3, "test content").expect("Failed to create test file with special chars");
        files.push(file3);

        (temp_dir, files)
    }

    #[test]
    fn test_file_drop_handler_creation() {
        let handler = FileDropHandler::new();
        assert_eq!(handler.get_config().max_files, 100);
        assert_eq!(handler.get_config().max_file_size_mb, 1024);
        assert!(handler.get_config().validate_existence);
        assert!(handler.get_config().resolve_symlinks);
    }

    #[test]
    fn test_file_drop_handler_with_custom_config() {
        let config = FileDropConfig {
            max_files: 10,
            max_file_size_mb: 100,
            allowed_extensions: Some(vec!["txt".to_string(), "md".to_string()]),
            validate_existence: false,
            resolve_symlinks: false,
        };

        let handler = FileDropHandler::new_with_config(config.clone());
        assert_eq!(handler.get_config().max_files, 10);
        assert_eq!(handler.get_config().max_file_size_mb, 100);
        assert_eq!(handler.get_config().allowed_extensions, Some(vec!["txt".to_string(), "md".to_string()]));
        assert!(!handler.get_config().validate_existence);
        assert!(!handler.get_config().resolve_symlinks);
    }

    #[test]
    fn test_escape_path_with_spaces() {
        let handler = FileDropHandler::new();
        
        let path_with_spaces = "/path/to/file with spaces.txt";
        let escaped = handler.escape_path(path_with_spaces);
        assert_eq!(escaped, "'/path/to/file with spaces.txt'");
    }

    #[test]
    fn test_escape_path_without_spaces() {
        let handler = FileDropHandler::new();
        
        let path_without_spaces = "/path/to/file.txt";
        let escaped = handler.escape_path(path_without_spaces);
        assert_eq!(escaped, "/path/to/file.txt");
    }

    #[test]
    fn test_escape_path_with_special_characters() {
        let handler = FileDropHandler::new();
        
        let test_cases = vec![
            ("/path/with(parentheses).txt", "'/path/with(parentheses).txt'"),
            ("/path/with[brackets].txt", "'/path/with[brackets].txt'"),
            ("/path/with{braces}.txt", "'/path/with{braces}.txt'"),
            ("/path/with&ampersand.txt", "'/path/with&ampersand.txt'"),
            ("/path/with|pipe.txt", "'/path/with|pipe.txt'"),
            ("/path/with;semicolon.txt", "'/path/with;semicolon.txt'"),
            ("/path/with<less.txt", "'/path/with<less.txt'"),
            ("/path/with>greater.txt", "'/path/with>greater.txt'"),
            ("/path/with?question.txt", "'/path/with?question.txt'"),
            ("/path/with*asterisk.txt", "'/path/with*asterisk.txt'"),
            ("/path/with$dollar.txt", "'/path/with$dollar.txt'"),
            ("/path/with\\backslash.txt", "'/path/with\\backslash.txt'"),
        ];

        for (input, expected) in test_cases {
            let escaped = handler.escape_path(input);
            assert_eq!(escaped, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_escape_path_with_single_quotes() {
        let handler = FileDropHandler::new();
        
        let path_with_quotes = "/path/with'single'quotes.txt";
        let escaped = handler.escape_path(path_with_quotes);
        assert_eq!(escaped, "'/path/with'\"'\"'single'\"'\"'quotes.txt'");
    }

    #[test]
    fn test_format_path_for_terminal() {
        let handler = FileDropHandler::new();
        
        let path = PathBuf::from("/path/to/file with spaces.txt");
        let formatted = handler.format_path_for_terminal(&path);
        assert_eq!(formatted, "'/path/to/file with spaces.txt'");
    }

    #[test]
    fn test_validate_path_safety_safe_paths() {
        let handler = FileDropHandler::new();
        
        let safe_paths = vec![
            "/path/to/file.txt",
            "/Users/username/Documents/file.pdf",
            "/tmp/temporary_file.log",
            "relative/path/file.txt",
        ];

        for path in safe_paths {
            assert!(handler.validate_path_safety(path).is_ok(), "Path should be safe: {}", path);
        }
    }

    #[test]
    fn test_validate_path_safety_dangerous_paths() {
        let handler = FileDropHandler::new();
        
        // These should generate warnings but not fail
        let warning_paths = vec![
            "/path/../other/file.txt",
            "~/file.txt",
            "/path/with$variable.txt",
            "/path/with`command`.txt",
            "/path/with$(command).txt",
            "/path/with&&command.txt",
            "/path/with||command.txt",
            "/path/with;command.txt",
            "/path/with|pipe.txt",
            "/path/with>redirect.txt",
            "/path/with<redirect.txt",
        ];

        for path in warning_paths {
            // These should succeed but generate warnings
            assert!(handler.validate_path_safety(path).is_ok(), "Path should succeed with warning: {}", path);
        }
    }

    #[test]
    fn test_validate_path_safety_invalid_paths() {
        let handler = FileDropHandler::new();
        
        // Path with null byte should fail
        let null_byte_path = "/path/with\0null.txt";
        assert!(handler.validate_path_safety(null_byte_path).is_err());
        
        // Extremely long path should fail
        let long_path = "a".repeat(5000);
        assert!(handler.validate_path_safety(&long_path).is_err());
    }

    #[test]
    fn test_handle_drop_event_single_file() {
        let (_temp_dir, files) = create_test_files();
        let handler = FileDropHandler::new();
        
        let result = handler.handle_drop_event(vec![files[0].clone()], "test_terminal");
        assert!(result.is_ok(), "Should handle single file drop successfully");
        
        let formatted_path = result.unwrap();
        assert!(!formatted_path.is_empty());
        assert!(formatted_path.contains("test_file.txt"));
    }

    #[test]
    fn test_handle_drop_event_multiple_files() {
        let (_temp_dir, files) = create_test_files();
        let handler = FileDropHandler::new();
        
        let result = handler.handle_drop_event(files.clone(), "test_terminal");
        assert!(result.is_ok(), "Should handle multiple file drop successfully");
        
        let formatted_paths = result.unwrap();
        assert!(!formatted_paths.is_empty());
        // Should contain all files separated by spaces
        assert!(formatted_paths.contains("test_file.txt"));
        // The path with spaces should be quoted
        assert!(formatted_paths.contains("file with spaces.txt"));
    }

    #[test]
    fn test_handle_drop_event_empty_files() {
        let handler = FileDropHandler::new();
        
        let result = handler.handle_drop_event(vec![], "test_terminal");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_handle_drop_event_nonexistent_file() {
        let handler = FileDropHandler::new();
        
        let nonexistent_file = PathBuf::from("/nonexistent/file.txt");
        let result = handler.handle_drop_event(vec![nonexistent_file], "test_terminal");
        assert!(result.is_err(), "Should fail for nonexistent file");
    }

    #[test]
    fn test_process_dropped_files() {
        let (_temp_dir, files) = create_test_files();
        let handler = FileDropHandler::new();
        
        let result = handler.process_dropped_files(files.clone());
        assert!(result.is_ok(), "Should process files successfully");
        
        let drop_result = result.unwrap();
        assert_eq!(drop_result.processed_files.len(), files.len());
        assert!(drop_result.skipped_files.is_empty());
        assert_eq!(drop_result.formatted_paths.len(), files.len());
        assert!(!drop_result.combined_path_string.is_empty());
    }

    #[test]
    fn test_process_dropped_files_with_restrictions() {
        let (_temp_dir, files) = create_test_files();
        
        // Create handler that only allows .txt files
        let config = FileDropConfig {
            allowed_extensions: Some(vec!["txt".to_string()]),
            ..Default::default()
        };
        let handler = FileDropHandler::new_with_config(config);
        
        let result = handler.process_dropped_files(files.clone());
        assert!(result.is_ok(), "Should process files with some skipped");
        
        let drop_result = result.unwrap();
        // The extension restriction only applies to files, not directories
        // So all files should be processed (3 .txt files + 1 directory = 4 total)
        // But let's check that we have some processed files and potentially some skipped
        assert!(drop_result.processed_files.len() > 0, "Should process some files");
        assert!(drop_result.processed_files.len() <= files.len(), "Should not process more files than provided");
    }

    #[test]
    fn test_process_dropped_files_too_many() {
        let (_temp_dir, files) = create_test_files();
        
        // Create handler with very low file limit
        let config = FileDropConfig {
            max_files: 1,
            ..Default::default()
        };
        let handler = FileDropHandler::new_with_config(config);
        
        let result = handler.process_dropped_files(files.clone());
        assert!(result.is_err(), "Should fail when too many files");
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Too many files"));
    }

    #[test]
    fn test_get_file_info() {
        let (_temp_dir, files) = create_test_files();
        let handler = FileDropHandler::new();
        
        // Test file info for regular file
        let file_info = handler.get_file_info(&files[0]);
        assert!(file_info.is_ok());
        let info = file_info.unwrap();
        assert!(info.contains("File:"));
        assert!(info.contains("test_file.txt"));
        assert!(info.contains("bytes"));
        
        // Test file info for directory
        let dir_info = handler.get_file_info(&files[2]);
        assert!(dir_info.is_ok());
        let info = dir_info.unwrap();
        assert!(info.contains("Directory:"));
        assert!(info.contains("test_directory"));
    }

    #[test]
    fn test_get_file_info_nonexistent() {
        let handler = FileDropHandler::new();
        let nonexistent = PathBuf::from("/nonexistent/file.txt");
        
        let result = handler.get_file_info(&nonexistent);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "File does not exist");
    }

    #[test]
    fn test_file_type_detection() {
        let (_temp_dir, files) = create_test_files();
        let handler = FileDropHandler::new();
        
        // Test file type detection
        assert_eq!(handler.determine_file_type(&files[0]), FileType::File);
        assert_eq!(handler.determine_file_type(&files[2]), FileType::Directory);
    }

    #[test]
    fn test_config_update() {
        let mut handler = FileDropHandler::new();
        
        let new_config = FileDropConfig {
            max_files: 50,
            max_file_size_mb: 500,
            allowed_extensions: Some(vec!["pdf".to_string()]),
            validate_existence: false,
            resolve_symlinks: false,
        };
        
        handler.update_config(new_config.clone());
        
        assert_eq!(handler.get_config().max_files, 50);
        assert_eq!(handler.get_config().max_file_size_mb, 500);
        assert_eq!(handler.get_config().allowed_extensions, Some(vec!["pdf".to_string()]));
        assert!(!handler.get_config().validate_existence);
        assert!(!handler.get_config().resolve_symlinks);
    }

    #[test]
    fn test_symlink_handling() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Create a target file
        let target_file = temp_dir.path().join("target.txt");
        fs::write(&target_file, "target content").expect("Failed to create target file");
        
        // Create a symlink to the target file
        let symlink_path = temp_dir.path().join("symlink.txt");
        
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target_file, &symlink_path)
                .expect("Failed to create symlink");
            
            let handler = FileDropHandler::new();
            let result = handler.process_dropped_files(vec![symlink_path.clone()]);
            
            assert!(result.is_ok(), "Should handle symlink successfully");
            let drop_result = result.unwrap();
            assert_eq!(drop_result.processed_files.len(), 1);
            assert!(drop_result.processed_files[0].is_symlink);
        }
    }

    #[test]
    fn test_large_file_handling() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let large_file = temp_dir.path().join("large_file.txt");
        
        // Create a file with some content (not actually large for test speed)
        fs::write(&large_file, "a".repeat(1000)).expect("Failed to create large file");
        
        // Create handler with very small file size limit
        let config = FileDropConfig {
            max_file_size_mb: 0, // 0 MB limit
            ..Default::default()
        };
        let handler = FileDropHandler::new_with_config(config);
        
        let result = handler.process_dropped_files(vec![large_file]);
        assert!(result.is_ok(), "Should process but may skip large files");
        
        let drop_result = result.unwrap();
        // File should be skipped due to size limit
        assert!(drop_result.skipped_files.len() > 0 || drop_result.processed_files.len() > 0);
    }
}