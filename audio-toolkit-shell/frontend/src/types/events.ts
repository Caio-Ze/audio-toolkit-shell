// Event types for Tauri events

export interface TerminalOutputEvent {
  terminal_id: string;
  line: string;        // Backend sends "line", not "output"
  stream: string;      // Backend sends "stream" ("stdout"/"stderr"), not "type"
  timestamp: string;
}

export interface ProcessStatusChangedEvent {
  terminal_id: string;
  status: string;
  pid?: number;
  timestamp: string;
  error?: string;
}

export interface FileDroppedEvent {
  terminal_id: string;
  file_paths: string[];
  timestamp: string;
}

// File drop related types
export interface FileDropConfig {
  max_files: number;
  max_file_size_mb: number;
  allowed_extensions?: string[];
  validate_existence: boolean;
  resolve_symlinks: boolean;
}

export type FileType = 'File' | 'Directory' | 'Symlink' | 'Unknown';

export interface DroppedFile {
  original_path: string;
  resolved_path: string;
  formatted_path: string;
  file_type: FileType;
  size_bytes?: number;
  is_symlink: boolean;
}

export interface FileDropResult {
  processed_files: DroppedFile[];
  skipped_files: Array<{
    path: string;
    reason: string;
  }>;
  formatted_paths: string[];
  combined_path_string: string;
}

export interface FileValidationResult {
  path: string;
  is_valid: boolean;
  message: string;
  file_info?: string;
}

// Event names as constants
export const EVENT_NAMES = {
  TERMINAL_OUTPUT: 'terminal-output',
  PROCESS_STATUS_CHANGED: 'process-status-changed',
  FILE_DROPPED: 'file-dropped',
} as const;

export type EventName = typeof EVENT_NAMES[keyof typeof EVENT_NAMES];