export interface TerminalConfig {
  id: string;
  name: string;
  launcher_executable: string;
  working_directory: string;
  environment_variables: Record<string, string>;
  auto_start: boolean;
}

export interface TerminalState {
  config: TerminalConfig;
  is_active: boolean;
  last_output: string;
  process_id?: number;
}