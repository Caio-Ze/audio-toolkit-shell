// Utility functions for the application

use std::collections::HashMap;

use crate::models::TerminalConfig;

/// Return the hard-coded launcher configurations.
/// Per user directive, development mode ALWAYS uses the real executables.
pub fn get_default_terminal_configs() -> Vec<TerminalConfig> {
    vec![
        TerminalConfig {
            id: "start_scripts_rust".into(),
            name: "Start Scripts (Rust)".into(),
            launcher_executable: "/Users/caioraphael/PYTHON/FULL_APP/launchers/audio-analyzer/start_scripts/PYTHON_SCRIPTS/start_scripts_rust".into(),
            working_directory: "/Users/caioraphael/PYTHON/FULL_APP/launchers/audio-analyzer/start_scripts/PYTHON_SCRIPTS".into(),
            environment_variables: HashMap::new(),
            auto_start: true,
        },
        TerminalConfig {
            id: "audio_normalizer".into(),
            name: "Audio Normalizer".into(),
            launcher_executable: "/Users/caioraphael/PYTHON/FULL_APP/launchers/audio-analyzer/start_scripts/RUST_SUPERFASTNORMALIZER/audio-normalizer-interactive".into(),
            working_directory: "/Users/caioraphael/PYTHON/FULL_APP/launchers/audio-analyzer/start_scripts/RUST_SUPERFASTNORMALIZER".into(),
            environment_variables: HashMap::new(),
            auto_start: true,
        },
        TerminalConfig {
            id: "session_monitor".into(),
            name: "Session Monitor".into(),
            launcher_executable: "/Users/caioraphael/PYTHON/FULL_APP/launchers/audio-analyzer/start_scripts/PYTHON_SCRIPTS/SCRIPTS_PYTHON/MONITOR/session-monitor".into(),
            working_directory: "/Users/caioraphael/PYTHON/FULL_APP/launchers/audio-analyzer/start_scripts/PYTHON_SCRIPTS/SCRIPTS_PYTHON/MONITOR".into(),
            environment_variables: HashMap::new(),
            auto_start: true,
        },
        TerminalConfig {
            id: "ptsl_launcher".into(),
            name: "Pro Tools Session Launcher".into(),
            launcher_executable: "/Users/caioraphael/PYTHON/FULL_APP/launchers/audio-analyzer/start_scripts/PYTHON_SCRIPTS/SCRIPTS_PYTHON/ptsl-launcher".into(),
            working_directory: "/Users/caioraphael/PYTHON/FULL_APP/launchers/audio-analyzer/start_scripts/PYTHON_SCRIPTS/SCRIPTS_PYTHON".into(),
            environment_variables: HashMap::new(),
            auto_start: true,
        },
    ]
}
