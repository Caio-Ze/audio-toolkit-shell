// Type definitions for the Audio Toolkit Shell application

export interface TerminalConfig {
  id: string;
  name: string;
  launcher_executable: string;
  working_directory: string;
  environment_variables: Record<string, string>;
  auto_start: boolean;
}

export interface ProcessInfo {
  terminal_id: string;
  status: ProcessStatus;
  pid?: number;
  started_at?: string;
  last_activity?: string;
}

export type ProcessStatus = 
  | 'Starting'
  | 'Running' 
  | 'Idle'
  | 'Processing'
  | 'Terminated'
  | { Error: string };

export interface AuthStatus {
  is_authenticated: boolean;
  user_id?: string;
  permissions?: string[];
}

export interface VersionStatus {
  current_version: string;
  latest_version?: string;
  update_required?: boolean;
}

export interface FileDropEvent {
  paths: string[];
  position?: { x: number; y: number };
}

export interface TerminalOutputEvent {
  terminal_id: string;
  line: string;
  stream: 'stdout' | 'stderr';
  timestamp: string;
}

export interface ProcessStatusEvent {
  terminal_id: string;
  status: ProcessStatus;
  timestamp: string;
}
