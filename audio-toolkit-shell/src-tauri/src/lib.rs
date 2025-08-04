// Modules
mod models;
mod services;
mod handlers;
mod utils;

#[cfg(test)]
mod tests;

use handlers::*;
use services::ProcessManager;
use tauri::{Manager, Listener, Emitter};
use tauri_plugin_pty;
use chrono;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_pty::init())
    .setup(|app| {
      // Initialize logging for debug builds
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Initialize the ProcessManager and manage its state
      let app_handle = app.handle().clone();
      let process_manager = ProcessManager::new(app_handle.clone());
      
      // Auto-spawn only start_scripts_rust for testing Tab 1
      let configs = utils::get_default_terminal_configs();
      for config in configs {
        if config.auto_start && config.id == "start_scripts_rust" {
          log::info!("🔥 SPAWNING ONLY start_scripts_rust for testing");
          
          // Check if executable exists and is executable
          let exe_path = std::path::Path::new(&config.launcher_executable);
          if !exe_path.exists() {
            log::error!("🔥 EXECUTABLE DOES NOT EXIST: {}", config.launcher_executable);
            continue;
          }
          
          log::info!("🔥 EXECUTABLE EXISTS: {}", config.launcher_executable);
          
          if let Err(e) = process_manager.spawn_launcher(&config) {
            log::error!("Failed to auto-spawn launcher '{}': {}", config.name, e);
          } else {
            log::info!("Successfully auto-spawned launcher: '{}'", config.name);
          }
        }
      }
      
      // Manage the ProcessManager state
      app.manage(process_manager);

      // Set up PTY event listeners - SIMPLIFIED for debugging
      let app_handle_for_pty = app.handle().clone();
      
      log::info!("🔥 Setting up PTY event listeners...");
      
      // Listen for ALL possible PTY events to debug what's actually being emitted
      let possible_events = vec![
          "pty-output", "pty_output", "pty-data", "pty_data", 
          "pty-stdout", "pty_stdout", "terminal-output", "terminal_output",
          "pty-spawn", "pty_spawn", "pty-write", "pty_write",
          "pty-exit", "pty_exit", "pty-resize", "pty_resize"
      ];
      
      for event_name in possible_events {
          let app_handle_clone = app_handle_for_pty.clone();
          let event_name_str = event_name.to_string();
          app.listen(event_name, move |event| {
              let payload = event.payload();
              log::info!("🔥🔥🔥 PTY EVENT '{}' received: {}", event_name_str, payload);
              
              // If this is an output event, try to forward it
              if event_name_str.contains("output") || event_name_str.contains("data") || event_name_str.contains("stdout") {
                  if let Ok(data) = serde_json::from_str::<serde_json::Value>(payload) {
                      if let (Some(terminal_id), Some(output)) = (
                          data.get("terminal_id").and_then(|v| v.as_str()),
                          data.get("data").and_then(|v| v.as_str())
                      ) {
                          log::info!("🔥 Forwarding output for terminal '{}': {}", terminal_id, output);
                          
                          let terminal_output = serde_json::json!({
                              "terminal_id": terminal_id,
                              "line": output,
                              "stream": "stdout",
                              "timestamp": chrono::Utc::now().to_rfc3339()
                          });
                          
                          if let Err(e) = app_handle_clone.emit("terminal-output", terminal_output) {
                              log::error!("Failed to emit terminal-output: {}", e);
                          }
                      }
                  }
              }
          });
      }
      
      log::info!("🔥 PTY event listeners set up complete");

      // Listen for PTY status changes
      let app_handle_for_status = app.handle().clone();
      app.listen("pty-exit", move |event| {
          let payload = event.payload();
          log::info!("PTY process exited: {}", payload);
          if let Ok(data) = serde_json::from_str::<serde_json::Value>(payload) {
              if let Some(terminal_id) = data.get("terminal_id").and_then(|v| v.as_str()) {
                  let status_change = serde_json::json!({
                      "terminal_id": terminal_id,
                      "status": "Exited",
                      "timestamp": chrono::Utc::now().to_rfc3339()
                  });
                  
                  if let Err(e) = app_handle_for_status.emit("process-status-changed", status_change) {
                      log::error!("Failed to emit process-status-changed: {}", e);
                  }
              }
          }
      });

      // Add a graceful shutdown hook to terminate all PTY processes on exit
      let app_handle_clone = app.handle().clone();
      app.listen("tauri://close-requested", move |_event| {
          log::info!("Close requested: terminating all running PTY processes.");
          let manager = app_handle_clone.state::<ProcessManager>();
          manager.shutdown_all_processes();
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      // --- Authentication ---
      validate_user_access,
      authenticate_with_key,
      // --- Configuration ---
              get_terminal_configs,
        get_all_process_statuses,
      // --- PTY Process Management ---
      spawn_launcher,
      send_terminal_input,
      terminate_process,
      // --- File Handling ---
      handle_file_drop,
      validate_file_paths,
      get_file_drop_info,
      format_path_for_terminal
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
