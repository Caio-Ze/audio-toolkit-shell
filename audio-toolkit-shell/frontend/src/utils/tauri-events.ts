import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type { 
  TerminalOutputEvent, 
  ProcessStatusChangedEvent
} from '../types/events';
import { EVENT_NAMES } from '../types/events';

/**
 * Listen for terminal output events
 * @param callback Function to handle terminal output
 * @returns Promise that resolves to an unlisten function
 */
export async function listenToTerminalOutput(
  callback: (event: TerminalOutputEvent) => void
): Promise<UnlistenFn> {
  return await listen<TerminalOutputEvent>(EVENT_NAMES.TERMINAL_OUTPUT, (event) => {
    callback(event.payload);
  });
}

/**
 * Listen for process status change events
 * @param callback Function to handle process status changes
 * @returns Promise that resolves to an unlisten function
 */
export async function listenToProcessStatusChanges(
  callback: (event: ProcessStatusChangedEvent) => void
): Promise<UnlistenFn> {
  return await listen<ProcessStatusChangedEvent>(EVENT_NAMES.PROCESS_STATUS_CHANGED, (event) => {
    callback(event.payload);
  });
}

/**
 * Listen for terminal output from a specific terminal
 * @param terminalId The ID of the terminal to listen to
 * @param callback Function to handle terminal output
 * @returns Promise that resolves to an unlisten function
 */
export async function listenToTerminalOutputForTerminal(
  terminalId: string,
  callback: (output: string, timestamp: string, isStderr?: boolean) => void
): Promise<UnlistenFn> {
  return await listenToTerminalOutput((event) => {
    if (event.terminal_id === terminalId) {
      callback(event.line, event.timestamp, event.stream === 'stderr');
    }
  });
}

/**
 * Listen for process status changes for a specific terminal
 * @param terminalId The ID of the terminal to listen to
 * @param callback Function to handle process status changes
 * @returns Promise that resolves to an unlisten function
 */
export async function listenToProcessStatusForTerminal(
  terminalId: string,
  callback: (status: string, pid?: number, error?: string) => void
): Promise<UnlistenFn> {
  return await listenToProcessStatusChanges((event) => {
    if (event.terminal_id === terminalId) {
      callback(event.status, event.pid, event.error);
    }
  });
}

/**
 * Set up all event listeners for a terminal
 * @param terminalId The ID of the terminal
 * @param handlers Object containing event handlers
 * @returns Promise that resolves to an object with unlisten functions
 */
export async function setupTerminalEventListeners(
  terminalId: string,
  handlers: {
    onOutput?: (output: string, timestamp: string, isStderr?: boolean) => void;
    onStatusChange?: (status: string, pid?: number, error?: string) => void;
  }
): Promise<{
  unlistenOutput?: UnlistenFn;
  unlistenStatus?: UnlistenFn;
}> {
  const result: {
    unlistenOutput?: UnlistenFn;
    unlistenStatus?: UnlistenFn;
  } = {};

  if (handlers.onOutput) {
    result.unlistenOutput = await listenToTerminalOutputForTerminal(
      terminalId,
      handlers.onOutput
    );
  }

  if (handlers.onStatusChange) {
    result.unlistenStatus = await listenToProcessStatusForTerminal(
      terminalId,
      handlers.onStatusChange
    );
  }

  return result;
}

/**
 * Clean up event listeners
 * @param listeners Object containing unlisten functions
 */
export function cleanupEventListeners(listeners: {
  unlistenOutput?: UnlistenFn;
  unlistenStatus?: UnlistenFn;
}) {
  if (listeners.unlistenOutput) {
    listeners.unlistenOutput();
  }
  if (listeners.unlistenStatus) {
    listeners.unlistenStatus();
  }
}