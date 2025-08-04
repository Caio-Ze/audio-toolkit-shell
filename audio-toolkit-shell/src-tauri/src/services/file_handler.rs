use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone)]
pub struct FileDropConfig {
    pub max_files: usize,
    pub max_file_size_mb: u64,
    pub allowed_extensions: Option<Vec<String>>,
    pub validate_existence: bool,
    pub resolve_symlinks: bool,
}

impl Default for FileDropConfig {
    fn default() -> Self {
        Self {
            max_files: 100,
            max_file_size_mb: 1024, // 1GB
            allowed_extensions: None, // Allow all extensions
            validate_existence: true,
            resolve_symlinks: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DroppedFile {
    pub original_path: PathBuf,
    pub resolved_path: PathBuf,
    pub formatted_path: String,
    pub file_type: FileType,
    pub size_bytes: Option<u64>,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FileDropResult {
    pub processed_files: Vec<DroppedFile>,
    pub skipped_files: Vec<(PathBuf, String)>, // Path and reason for skipping
    pub formatted_paths: Vec<String>,
    pub combined_path_string: String,
}

pub struct FileDropHandler {
    config: FileDropConfig,
}

impl FileDropHandler {
    pub fn new() -> Self {
        Self {
            config: FileDropConfig::default(),
        }
    }

    pub fn new_with_config(config: FileDropConfig) -> Self {
        Self { config }
    }

    /// Main method for handling file drop events with comprehensive validation
    pub fn handle_drop_event(&self, files: Vec<PathBuf>, terminal_id: &str) -> Result<String> {
        log::info!("Handling file drop for terminal: {} with {} files", terminal_id, files.len());
        
        if files.is_empty() {
            return Ok(String::new());
        }

        let result = self.process_dropped_files(files)?;
        
        if result.processed_files.is_empty() {
            return Err(anyhow!("No valid files to process"));
        }

        // Log any skipped files
        for (path, reason) in &result.skipped_files {
            log::warn!("Skipped file {}: {}", path.display(), reason);
        }

        Ok(result.combined_path_string)
    }

    /// Process multiple dropped files with validation and formatting
    pub fn process_dropped_files(&self, files: Vec<PathBuf>) -> Result<FileDropResult> {
        if files.len() > self.config.max_files {
            return Err(anyhow!(
                "Too many files dropped: {} (max: {})", 
                files.len(), 
                self.config.max_files
            ));
        }

        let mut processed_files = Vec::new();
        let mut skipped_files = Vec::new();
        let mut formatted_paths = Vec::new();

        for file_path in files {
            match self.process_single_file(&file_path) {
                Ok(dropped_file) => {
                    formatted_paths.push(dropped_file.formatted_path.clone());
                    processed_files.push(dropped_file);
                }
                Err(e) => {
                    skipped_files.push((file_path, e.to_string()));
                }
            }
        }

        let combined_path_string = if formatted_paths.len() == 1 {
            formatted_paths[0].clone()
        } else {
            formatted_paths.join(" ")
        };

        Ok(FileDropResult {
            processed_files,
            skipped_files,
            formatted_paths,
            combined_path_string,
        })
    }

    /// Process a single file with comprehensive validation
    fn process_single_file(&self, file_path: &PathBuf) -> Result<DroppedFile> {
        // Validate file existence if required
        if self.config.validate_existence && !file_path.exists() {
            return Err(anyhow!("File does not exist: {}", file_path.display()));
        }

        let mut resolved_path = file_path.clone();
        let mut is_symlink = false;

        // Resolve symlinks if configured
        if self.config.resolve_symlinks && file_path.is_symlink() {
            is_symlink = true;
            resolved_path = fs::read_link(file_path)
                .map_err(|e| anyhow!("Failed to resolve symlink {}: {}", file_path.display(), e))?;
            
            // Make relative symlinks absolute
            if resolved_path.is_relative() {
                if let Some(parent) = file_path.parent() {
                    resolved_path = parent.join(resolved_path);
                }
            }
        }

        // Determine file type
        let file_type = self.determine_file_type(&resolved_path);

        // Get file size for regular files
        let size_bytes = if file_type == FileType::File {
            match fs::metadata(&resolved_path) {
                Ok(metadata) => {
                    let size = metadata.len();
                    // Check file size limit
                    if size > self.config.max_file_size_mb * 1024 * 1024 {
                        return Err(anyhow!(
                            "File too large: {} MB (max: {} MB)",
                            size / (1024 * 1024),
                            self.config.max_file_size_mb
                        ));
                    }
                    Some(size)
                }
                Err(_) => None,
            }
        } else {
            None
        };

        // Validate file extension if restrictions are configured
        if let Some(ref allowed_extensions) = self.config.allowed_extensions {
            if file_type == FileType::File {
                if let Some(extension) = resolved_path.extension() {
                    let ext_str = extension.to_string_lossy().to_lowercase();
                    if !allowed_extensions.iter().any(|allowed| allowed.to_lowercase() == ext_str) {
                        return Err(anyhow!(
                            "File extension '{}' not allowed. Allowed: {:?}",
                            ext_str,
                            allowed_extensions
                        ));
                    }
                } else if !allowed_extensions.is_empty() {
                    return Err(anyhow!("File has no extension, but extensions are restricted"));
                }
            }
        }

        // Format the path for terminal use
        let formatted_path = self.format_path_for_terminal(&resolved_path);

        Ok(DroppedFile {
            original_path: file_path.clone(),
            resolved_path,
            formatted_path,
            file_type,
            size_bytes,
            is_symlink,
        })
    }

    /// Determine the type of a file system entry
    pub fn determine_file_type(&self, path: &PathBuf) -> FileType {
        if path.is_symlink() {
            FileType::Symlink
        } else if path.is_dir() {
            FileType::Directory
        } else if path.is_file() {
            FileType::File
        } else {
            FileType::Unknown
        }
    }

    /// Format a path for terminal insertion with proper macOS formatting
    pub fn format_path_for_terminal(&self, path: &PathBuf) -> String {
        let path_str = path.to_string_lossy();
        self.escape_path(&path_str)
    }

    /// Escape special characters in paths for safe terminal use
    pub fn escape_path(&self, path: &str) -> String {
        // Characters that need escaping in shell contexts
        let special_chars = [' ', '(', ')', '[', ']', '{', '}', '&', '|', ';', '<', '>', '?', '*', '\'', '"', '`', '$', '\\'];
        
        if special_chars.iter().any(|&c| path.contains(c)) {
            // Use single quotes for better compatibility with various shells
            // Escape any single quotes in the path by ending the quoted string,
            // adding an escaped single quote, and starting a new quoted string
            let escaped = path.replace('\'', "'\"'\"'");
            format!("'{}'", escaped)
        } else {
            path.to_string()
        }
    }

    /// Validate that a path is safe for terminal use
    pub fn validate_path_safety(&self, path: &str) -> Result<()> {
        // Check for potentially dangerous patterns
        let dangerous_patterns = [
            "..", // Directory traversal
            "~", // Home directory expansion might be unexpected
            "$", // Variable expansion
            "`", // Command substitution
            "$(", // Command substitution
            "&&", // Command chaining
            "||", // Command chaining
            ";", // Command separator
            "|", // Pipe
            ">", // Redirection
            "<", // Redirection
        ];

        for pattern in &dangerous_patterns {
            if path.contains(pattern) {
                log::warn!("Potentially unsafe path pattern '{}' found in: {}", pattern, path);
                // Don't fail, just warn - let the user decide
            }
        }

        // Check for null bytes (definitely dangerous)
        if path.contains('\0') {
            return Err(anyhow!("Path contains null byte"));
        }

        // Check for extremely long paths
        if path.len() > 4096 {
            return Err(anyhow!("Path too long: {} characters", path.len()));
        }

        Ok(())
    }

    /// Get file information for display purposes
    pub fn get_file_info(&self, path: &PathBuf) -> Result<String> {
        if !path.exists() {
            return Ok("File does not exist".to_string());
        }

        let metadata = fs::metadata(path)?;
        let file_type = if path.is_dir() {
            "Directory"
        } else if path.is_file() {
            "File"
        } else if path.is_symlink() {
            "Symlink"
        } else {
            "Unknown"
        };

        let size = if metadata.is_file() {
            format!(" ({} bytes)", metadata.len())
        } else {
            String::new()
        };

        Ok(format!("{}: {}{}", file_type, path.display(), size))
    }

    /// Update configuration
    pub fn update_config(&mut self, config: FileDropConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &FileDropConfig {
        &self.config
    }
}

impl Default for FileDropHandler {
    fn default() -> Self {
        Self::new()
    }
}