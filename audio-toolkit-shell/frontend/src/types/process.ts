export type ProcessStatus = 
  | "Starting"
  | "Running" 
  | "Idle"
  | "Processing"
  | { Error: string }
  | "Terminated";

export interface ProcessInfo {
  terminal_id: string;
  status: ProcessStatus;
  pid?: number;
  started_at?: string;
  last_activity?: string;
  cpu_usage?: number;
  memory_usage?: number;
}