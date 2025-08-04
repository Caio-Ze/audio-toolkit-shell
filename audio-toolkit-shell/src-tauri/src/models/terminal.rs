use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerminalConfig {
    pub id: String,
    pub name: String,
    pub launcher_executable: String,
    pub working_directory: String,
    pub environment_variables: HashMap<String, String>,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalState {
    pub config: TerminalConfig,
    pub is_active: bool,
    pub last_output: String,
    pub process_id: Option<u32>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            launcher_executable: String::new(),
            working_directory: String::from("."),
            environment_variables: HashMap::new(),
            auto_start: true,
        }
    }
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            config: TerminalConfig::default(),
            is_active: false,
            last_output: String::new(),
            process_id: None,
        }
    }
}