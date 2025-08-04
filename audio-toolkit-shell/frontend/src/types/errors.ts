// Error types matching Rust CommandError

export interface CommandError {
  AuthenticationError?: { message: string };
  ProcessError?: { message: string };
  FileError?: { message: string };
  NetworkError?: { message: string };
  ValidationError?: { message: string };
  InternalError?: { message: string };
}

export type CommandResult<T> = {
  success: true;
  data: T;
} | {
  success: false;
  error: CommandError;
};

// Helper functions for error handling
export function isCommandError(result: any): result is { success: false; error: CommandError } {
  return result && result.success === false && result.error;
}

export function getErrorMessage(error: CommandError): string {
  if (error.AuthenticationError) return error.AuthenticationError.message;
  if (error.ProcessError) return error.ProcessError.message;
  if (error.FileError) return error.FileError.message;
  if (error.NetworkError) return error.NetworkError.message;
  if (error.ValidationError) return error.ValidationError.message;
  if (error.InternalError) return error.InternalError.message;
  return 'Unknown error occurred';
}

export function getErrorType(error: CommandError): string {
  if (error.AuthenticationError) return 'Authentication';
  if (error.ProcessError) return 'Process';
  if (error.FileError) return 'File';
  if (error.NetworkError) return 'Network';
  if (error.ValidationError) return 'Validation';
  if (error.InternalError) return 'Internal';
  return 'Unknown';
}