import { invoke } from '@tauri-apps/api/core'
import type { 
  TerminalConfig, 
  ProcessInfo
} from '../types'
import type { AuthResponse } from '../types/auth'

/**
 * Validate user access and get authentication status
 */
export async function validateUserAccess(): Promise<AuthResponse> {
  return await invoke<AuthResponse>('validate_user_access')
}

/**
 * Send input to a terminal
 */
export async function sendTerminalInput(terminalId: string, input: string): Promise<void> {
  await invoke('send_terminal_input', {
    terminalId: terminalId,
    input
  })
}

/**
 * Get terminal output
 */
export async function getTerminalOutput(terminalId: string): Promise<string> {
  return await invoke<string>('get_terminal_output', {
    terminal_id: terminalId
  })
}

/**
 * Handle file drop on terminal
 */
export async function handleFileDrop(terminalId: string, filePaths: string[]): Promise<void> {
  await invoke('handle_file_drop', {
    terminal_id: terminalId,
    file_paths: filePaths
  })
}

/**
 * Get terminal configurations
 */
export async function getTerminalConfigs(): Promise<TerminalConfig[]> {
  try {
    return await invoke<TerminalConfig[]>('get_terminal_configs')
  } catch (error) {
    // Fallback to mock data for development
    console.warn('Failed to get terminal configs from backend, using mock data:', error)
    return [
      {
        id: 'super-fast-normalizer',
        name: 'Super Fast Normalizer',
        launcher_executable: 'super-fast-normalizer',
        working_directory: '/path/to/normalizer',
        environment_variables: {},
        auto_start: true
      },
      {
        id: 'start-scripts',
        name: 'Start Scripts',
        launcher_executable: 'start-scripts',
        working_directory: '/path/to/scripts',
        environment_variables: {},
        auto_start: true
      },
      {
        id: 'audio-analyzer',
        name: 'Audio Analyzer',
        launcher_executable: 'audio-analyzer',
        working_directory: '/path/to/analyzer',
        environment_variables: {},
        auto_start: true
      },
      {
        id: 'batch-processor',
        name: 'Batch Processor',
        launcher_executable: 'batch-processor',
        working_directory: '/path/to/processor',
        environment_variables: {},
        auto_start: true
      },
      {
        id: 'format-converter',
        name: 'Format Converter',
        launcher_executable: 'format-converter',
        working_directory: '/path/to/converter',
        environment_variables: {},
        auto_start: true
      }
    ]
  }
}

/**
 * Start a terminal process
 */
export async function startTerminalProcess(terminalId: string): Promise<void> {
  await invoke('start_terminal_process', {
    terminal_id: terminalId
  })
}

/**
 * Stop a terminal process
 */
export async function stopTerminalProcess(terminalId: string): Promise<void> {
  await invoke('stop_terminal_process', {
    terminal_id: terminalId
  })
}

/**
 * Get process information for all terminals
 */
export async function getProcessInfo(): Promise<ProcessInfo[]> {
  return await invoke<ProcessInfo[]>('get_process_info')
}

/**
 * Get process information for a specific terminal
 */
export async function getTerminalProcessInfo(terminalId: string): Promise<ProcessInfo> {
  return await invoke<ProcessInfo>('get_terminal_process_info', {
    terminal_id: terminalId
  })
}

/**
 * Restart a terminal process
 */
export async function restartTerminalProcess(terminalId: string): Promise<void> {
  await invoke('restart_terminal_process', {
    terminal_id: terminalId
  })
}

/**
 * Clear terminal output
 */
export async function clearTerminal(terminalId: string): Promise<void> {
  await invoke('clear_terminal', {
    terminal_id: terminalId
  })
}

/**
 * Resize terminal
 */
export async function resizeTerminal(terminalId: string, cols: number, rows: number): Promise<void> {
  await invoke('resize_terminal', {
    terminal_id: terminalId,
    cols,
    rows
  })
}

/**
 * Get application version
 */
export async function getAppVersion(): Promise<string> {
  return await invoke<string>('get_app_version')
}

/**
 * Check if terminal is ready
 */
export async function isTerminalReady(terminalId: string): Promise<boolean> {
  return await invoke<boolean>('is_terminal_ready', {
    terminal_id: terminalId
  })
}

/**
 * Execute a script by ID
 */
export async function executeScript(scriptId: number, scriptName: string, scriptType: string): Promise<string> {
  return await invoke<string>('execute_script', {
    script_id: scriptId,
    script_name: scriptName,
    script_type: scriptType
  })
}

/**
 * Generic error handler for Tauri commands
 */
export function handleTauriError(error: unknown): string {
  if (typeof error === 'string') {
    return error
  }
  
  if (error instanceof Error) {
    return error.message
  }
  
  return 'An unknown error occurred'
}

/**
 * Wrapper for Tauri commands with error handling
 */
export async function safeTauriInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<{ success: true; data: T } | { success: false; error: string }> {
  try {
    const data = await invoke<T>(command, args)
    return { success: true, data }
  } catch (error) {
    return { success: false, error: handleTauriError(error) }
  }
}