use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    Starting,
    Spawning,
    Running,
    Idle,
    Processing,
    Error(String),
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub terminal_id: String,
    pub status: ProcessStatus,
    pub pid: Option<u32>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
    pub cpu_usage: Option<f32>,
    pub memory_usage: Option<u64>,
}

impl Default for ProcessStatus {
    fn default() -> Self {
        ProcessStatus::Starting
    }
}

impl Default for ProcessInfo {
    fn default() -> Self {
        Self {
            terminal_id: String::new(),
            status: ProcessStatus::default(),
            pid: None,
            started_at: None,
            last_activity: None,
            cpu_usage: None,
            memory_usage: None,
        }
    }
}