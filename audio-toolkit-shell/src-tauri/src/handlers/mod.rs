pub mod errors;

use crate::models::*;
use crate::services::{FileDropHandler, ProcessManager};
use errors::{CommandError, CommandResult};
use serde_json;
use tauri::State;

// Authentication and system handlers

#[tauri::command]
pub async fn validate_user_access() -> CommandResult<AuthResponse> {
    log::info!("Bypassing user access validation as requested.");
    Ok(AuthResponse {
        auth_status: AuthStatus {
            is_authenticated: true,
            user_id: Some("dev_user".to_string()),
            expires_at: None,
            permissions: vec!["all".to_string()],
        },
        version_status: VersionStatus::Current,
        server_message: Some("Authentication bypassed for development.".to_string()),
    })
}

#[tauri::command]
pub async fn authenticate_with_key(_user_key: String) -> CommandResult<AuthResponse> {
    log::info!("Bypassing authentication with key as requested.");
    Ok(AuthResponse {
        auth_status: AuthStatus {
            is_authenticated: true,
            user_id: Some("dev_user".to_string()),
            expires_at: None,
            permissions: vec!["all".to_string()],
        },
        version_status: VersionStatus::Current,
        server_message: Some("Authentication bypassed for development.".to_string()),
    })
}

// Terminal configuration handlers

#[tauri::command]
pub fn get_terminal_configs() -> CommandResult<Vec<TerminalConfig>> {
    log::info!("Getting terminal configurations");
    Ok(crate::utils::get_default_terminal_configs())
}

#[tauri::command]
pub fn get_all_process_statuses() -> CommandResult<Vec<ProcessInfo>> {
    log::info!("Getting initial statuses for all launchers");
    let configs = crate::utils::get_default_terminal_configs();
    let statuses = configs
        .into_iter()
        .map(|config| ProcessInfo {
            terminal_id: config.id,
            status: ProcessStatus::Starting,
            pid: None,
            started_at: None,
            last_activity: None,
            cpu_usage: None,
            memory_usage: None,
        })
        .collect();
    Ok(statuses)
}

// PTY Process management handlers

#[tauri::command]
pub fn spawn_launcher(
    config: TerminalConfig,
    process_manager: State<'_, ProcessManager>,
) -> CommandResult<ProcessInfo> {
    log::info!("Spawning launcher via PTY: {}", config.name);
    if config.id.trim().is_empty() {
        return Err(CommandError::validation("Terminal ID cannot be empty"));
    }
    if config.launcher_executable.trim().is_empty() {
        return Err(CommandError::validation("Launcher executable cannot be empty"));
    }
    process_manager.spawn_launcher(&config).map_err(|e| {
        log::error!("Failed to spawn PTY launcher {}: {}", config.name, e);
        CommandError::process(format!("Failed to spawn PTY launcher: {}", e))
    })
}

#[tauri::command]
pub fn send_terminal_input(
    terminal_id: String,
    input: String,
    process_manager: State<'_, ProcessManager>,
) -> CommandResult<()> {
    log::info!("Sending input to PTY {}: {}", terminal_id, input);
    if terminal_id.trim().is_empty() {
        return Err(CommandError::validation("Terminal ID cannot be empty"));
    }
        process_manager.send_terminal_input(&terminal_id, &input).map_err(|e| {
        log::error!("Failed to send input to PTY {}: {}", terminal_id, e);
        CommandError::process(format!("Failed to send input to PTY: {}", e))
    })
}

#[tauri::command]
pub fn terminate_process(
    terminal_id: String,
    _process_manager: State<ProcessManager>,
) -> CommandResult<String> {
    log::info!("Terminating PTY process: {}", terminal_id);
    if terminal_id.trim().is_empty() {
        return Err(CommandError::validation("Terminal ID cannot be empty"));
    }
    // This command is deprecated. The new shutdown logic is handled globally.
    log::warn!("The 'terminate_process' command is deprecated and will be removed.");
    Ok(format!("Termination request for {} is deprecated.", terminal_id))
}

// File handling

#[tauri::command]
pub fn handle_file_drop(
    terminal_id: String,
    file_paths: Vec<String>,
    process_manager: State<'_, ProcessManager>,
) -> CommandResult<()> {
    log::info!("Handling file drop for terminal {}: {:?}", terminal_id, file_paths);
    if terminal_id.is_empty() {
        return Err(CommandError::validation("Terminal ID cannot be empty."));
    }
    if file_paths.is_empty() {
        return Err(CommandError::validation("No file paths provided."));
    }

    let file_handler = FileDropHandler::new();
            let paths = file_paths.into_iter().map(std::path::PathBuf::from).collect();
    let result = file_handler.process_dropped_files(paths).map_err(|e| {
        CommandError::file(format!("Error processing files: {}", e))
    })?;

    if !result.skipped_files.is_empty() {
        log::warn!("Skipped {} invalid files.", result.skipped_files.len());
    }

    if !result.combined_path_string.is_empty() {
        log::info!("Sending formatted path string to terminal {}", terminal_id);
                process_manager.send_terminal_input(&terminal_id, &result.combined_path_string).map_err(|e| {
            CommandError::process(format!("Failed to send file paths to terminal: {}", e))
        })
    } else {
        Err(CommandError::file("All dropped files were invalid or inaccessible.".to_string()))
    }
}

#[tauri::command]
pub fn validate_file_paths(file_paths: Vec<String>) -> CommandResult<Vec<String>> {
    log::debug!("Validating file paths: {:?}", file_paths);
    if file_paths.is_empty() {
        return Err(CommandError::validation("No file paths provided for validation"));
    }

    let file_handler = FileDropHandler::new();
    let mut safe_paths = Vec::new();
    let mut errors = Vec::new();

    for path in file_paths {
        if let Err(e) = file_handler.validate_path_safety(&path) {
            errors.push(format!("Path '{}' is unsafe: {}", path, e));
        }
        
        let p = std::path::Path::new(&path);
        if !p.exists() {
            errors.push(format!("Path '{}' does not exist.", path));
        } else {
            safe_paths.push(path);
        }
    }

    if errors.is_empty() {
        Ok(safe_paths)
    } else {
        Err(CommandError::validation(errors.join("; ")))
    }
}

#[tauri::command]
pub fn get_file_drop_info(file_paths: Vec<String>) -> CommandResult<serde_json::Value> {
    log::info!("Getting info for file drop: {:?}", file_paths);
    if file_paths.is_empty() {
        return Err(CommandError::validation("No file paths provided."));
    }
    let file_handler = FileDropHandler::new();
            let paths = file_paths.into_iter().map(std::path::PathBuf::from).collect();
    match file_handler.process_dropped_files(paths) {
        Ok(result) => {
                        let info = serde_json::json!({
                "total_files": result.processed_files.len() + result.skipped_files.len(),
                "valid_files_count": result.processed_files.len(),
                "combined_path_string": result.combined_path_string,
                "skipped_details": result.skipped_files.iter().map(|(path, reason)| {
                    serde_json::json!({ 
                        "path": path.to_string_lossy(),
                        "reason": reason
                    })
                }).collect::<Vec<_>>()
            });
            Ok(info)
        }
        Err(e) => Err(CommandError::file(format!("Failed to process files: {}", e))),
    }
}

#[tauri::command]
pub fn format_path_for_terminal(file_path: String) -> CommandResult<String> {
    log::debug!("Formatting path for terminal: {}", file_path);
    if file_path.trim().is_empty() {
        return Err(CommandError::validation("File path cannot be empty"));
    }
    let file_handler = FileDropHandler::new();
    let path_buf = std::path::PathBuf::from(&file_path);
    file_handler.validate_path_safety(&file_path)
        .map_err(|e| CommandError::validation(format!("Unsafe path: {}", e)))?;
    let formatted = file_handler.format_path_for_terminal(&path_buf);
    Ok(formatted)
}