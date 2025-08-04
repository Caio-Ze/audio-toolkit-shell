use tauri::{AppHandle, Emitter, Wry};
use crate::models::{ProcessInfo, ProcessStatus, TerminalConfig};
use anyhow::Result;
use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};


#[derive(Serialize, Clone)]
pub struct PtySpawnPayload {
    file: String,
    args: Vec<String>,
    cwd: Option<String>,
    cols: u16,
    rows: u16,
}

#[derive(Clone)]
pub struct ProcessManager {
    processes: Arc<Mutex<HashMap<String, ProcessInfo>>>,
    app_handle: AppHandle<Wry>,
}

impl ProcessManager {
    pub fn new(app_handle: AppHandle<Wry>) -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            app_handle,
        }
    }

    pub fn spawn_launcher(&self, config: &TerminalConfig) -> Result<ProcessInfo> {
        log::info!("Spawning launcher via PTY plugin: {}", config.name);

        // Spawn the actual executable, not bash
        let payload = PtySpawnPayload {
            file: config.launcher_executable.clone(),
            args: vec![],
            cwd: Some(config.working_directory.clone()),
            cols: 80,
            rows: 24,
        };

        // Emit the PTY spawn request - the plugin will handle the actual spawning
        self.app_handle.emit("pty-spawn", Some(serde_json::json!({
            "terminal_id": config.id,
            "payload": payload
        }))).unwrap();

        log::info!("Emitted pty-spawn event for terminal: {}", config.id);

        let now = Utc::now();
        let process_info = ProcessInfo {
            terminal_id: config.id.clone(),
            status: ProcessStatus::Running,
            pid: None, // Will be updated by PTY plugin events
            started_at: Some(now),
            last_activity: Some(now),
            ..Default::default()
        };

        self.processes.lock().unwrap().insert(config.id.clone(), process_info.clone());
        Ok(process_info)
    }

    pub fn send_terminal_input(&self, terminal_id: &str, input: &str) -> Result<()> {
        log::debug!("Sending input to PTY terminal '{}'", terminal_id);
        // Use the PTY plugin's write event
        self.app_handle.emit("pty-write", Some(serde_json::json!({ 
            "terminal_id": terminal_id, 
            "data": input 
        }))).unwrap();
        Ok(())
    }

    pub fn shutdown_all_processes(&self) {
        log::info!("Requesting shutdown of all terminal processes...");
        let pids: Vec<u32> = self.processes.lock().unwrap().values()
            .filter_map(|p| if p.status == ProcessStatus::Running { p.pid } else { None })
            .collect();
        
        if !pids.is_empty() {
                        self.app_handle.emit("pty_kill_request", Some(pids)).unwrap();
        }
    }

    pub fn get_process_status(&self, terminal_id: &str) -> Option<ProcessStatus> {
        self.processes.lock().unwrap().get(terminal_id).map(|info| info.status.clone())
    }
}