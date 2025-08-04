import { invoke } from '@tauri-apps/api/core';
import type { 
  FileDropResult, 
  FileValidationResult
} from '../types/events';

/**
 * Handle file drop event by sending files to a terminal
 * @param terminalId The ID of the terminal to send files to
 * @param filePaths Array of file paths to process
 * @returns Promise that resolves when files are processed
 */
export async function handleFileDrop(
  terminalId: string, 
  filePaths: string[]
): Promise<void> {
  try {
    await invoke('handle_file_drop', {
      terminal_id: terminalId,
      file_paths: filePaths
    });
  } catch (error) {
    throw new Error(`Failed to handle file drop: ${error}`);
  }
}

/**
 * Validate file paths before dropping
 * @param filePaths Array of file paths to validate
 * @returns Promise that resolves to validation results
 */
export async function validateFilePaths(
  filePaths: string[]
): Promise<FileValidationResult[]> {
  try {
    const results = await invoke<string[]>('validate_file_paths', {
      file_paths: filePaths
    });
    
    return filePaths.map((path, index) => ({
      path,
      is_valid: !results[index].includes('Error') && !results[index].includes('does not exist'),
      message: results[index],
      file_info: results[index].startsWith('Valid:') ? results[index].substring(7) : undefined
    }));
  } catch (error) {
    throw new Error(`Failed to validate file paths: ${error}`);
  }
}

/**
 * Get detailed information about files being dropped
 * @param filePaths Array of file paths to analyze
 * @returns Promise that resolves to detailed file drop information
 */
export async function getFileDropInfo(
  filePaths: string[]
): Promise<FileDropResult> {
  try {
    const result = await invoke<any>('get_file_drop_info', {
      file_paths: filePaths
    });
    
    return {
      processed_files: result.processed_files || [],
      skipped_files: result.skipped_details || [],
      formatted_paths: result.formatted_paths || [],
      combined_path_string: result.combined_path_string || ''
    };
  } catch (error) {
    throw new Error(`Failed to get file drop info: ${error}`);
  }
}

/**
 * Format a single path for terminal use
 * @param filePath The file path to format
 * @returns Promise that resolves to the formatted path
 */
export async function formatPathForTerminal(filePath: string): Promise<string> {
  try {
    return await invoke<string>('format_path_for_terminal', {
      file_path: filePath
    });
  } catch (error) {
    throw new Error(`Failed to format path: ${error}`);
  }
}

/**
 * Extract file paths from drag and drop event
 * @param event The drag and drop event
 * @returns Array of file paths
 */
export function extractFilePathsFromDropEvent(event: DragEvent): string[] {
  const files: string[] = [];
  
  if (event.dataTransfer?.files) {
    for (let i = 0; i < event.dataTransfer.files.length; i++) {
      const file = event.dataTransfer.files[i];
      // Note: In a real Tauri app, you'd need to handle file paths differently
      // This is a simplified version for demonstration
      if ((file as any).path) {
        files.push((file as any).path);
      }
    }
  }
  
  return files;
}

/**
 * Validate that files are suitable for dropping
 * @param files Array of File objects from drag event
 * @returns Object with validation results
 */
export function validateDroppedFiles(files: File[]): {
  valid: boolean;
  errors: string[];
  warnings: string[];
} {
  const errors: string[] = [];
  const warnings: string[] = [];
  
  if (files.length === 0) {
    errors.push('No files selected');
    return { valid: false, errors, warnings };
  }
  
  if (files.length > 100) {
    errors.push(`Too many files: ${files.length} (maximum: 100)`);
  }
  
  for (const file of files) {
    // Check file size (1GB limit)
    if (file.size > 1024 * 1024 * 1024) {
      warnings.push(`Large file: ${file.name} (${Math.round(file.size / (1024 * 1024))} MB)`);
    }
    
    // Check for potentially problematic file names
    if (file.name.includes('..')) {
      warnings.push(`Potentially unsafe file name: ${file.name}`);
    }
    
    if (file.name.length > 255) {
      warnings.push(`Very long file name: ${file.name.substring(0, 50)}...`);
    }
  }
  
  return {
    valid: errors.length === 0,
    errors,
    warnings
  };
}

/**
 * Create a visual feedback element for drag and drop
 * @param element The element to add feedback to
 * @param isValidTarget Whether this is a valid drop target
 */
export function addDropFeedback(element: HTMLElement, isValidTarget: boolean = true): void {
  element.classList.add('drag-over');
  
  if (isValidTarget) {
    element.classList.add('valid-drop-target');
    element.style.backgroundColor = 'rgba(0, 255, 0, 0.1)';
    element.style.border = '2px dashed #00ff00';
  } else {
    element.classList.add('invalid-drop-target');
    element.style.backgroundColor = 'rgba(255, 0, 0, 0.1)';
    element.style.border = '2px dashed #ff0000';
  }
}

/**
 * Remove visual feedback from drag and drop
 * @param element The element to remove feedback from
 */
export function removeDropFeedback(element: HTMLElement): void {
  element.classList.remove('drag-over', 'valid-drop-target', 'invalid-drop-target');
  element.style.backgroundColor = '';
  element.style.border = '';
}

/**
 * Set up drag and drop event listeners for an element
 * @param element The element to set up drag and drop for
 * @param onDrop Callback for when files are dropped
 * @param onDragOver Optional callback for drag over events
 * @returns Function to remove event listeners
 */
export function setupDragAndDrop(
  element: HTMLElement,
  onDrop: (filePaths: string[]) => void,
  onDragOver?: (event: DragEvent) => void
): () => void {
  const handleDragOver = (event: DragEvent) => {
    event.preventDefault();
    event.stopPropagation();
    
    addDropFeedback(element);
    
    if (onDragOver) {
      onDragOver(event);
    }
  };
  
  const handleDragLeave = (event: DragEvent) => {
    event.preventDefault();
    event.stopPropagation();
    
    // Only remove feedback if we're actually leaving the element
    if (!element.contains(event.relatedTarget as Node)) {
      removeDropFeedback(element);
    }
  };
  
  const handleDrop = (event: DragEvent) => {
    event.preventDefault();
    event.stopPropagation();
    
    removeDropFeedback(element);
    
    const filePaths = extractFilePathsFromDropEvent(event);
    if (filePaths.length > 0) {
      onDrop(filePaths);
    }
  };
  
  element.addEventListener('dragover', handleDragOver);
  element.addEventListener('dragleave', handleDragLeave);
  element.addEventListener('drop', handleDrop);
  
  // Return cleanup function
  return () => {
    element.removeEventListener('dragover', handleDragOver);
    element.removeEventListener('dragleave', handleDragLeave);
    element.removeEventListener('drop', handleDrop);
    removeDropFeedback(element);
  };
}

/**
 * Format file size for display
 * @param bytes File size in bytes
 * @returns Formatted file size string
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

/**
 * Get file extension from path
 * @param filePath The file path
 * @returns File extension (without dot) or empty string
 */
export function getFileExtension(filePath: string): string {
  const lastDot = filePath.lastIndexOf('.');
  const lastSlash = Math.max(filePath.lastIndexOf('/'), filePath.lastIndexOf('\\'));
  
  if (lastDot > lastSlash && lastDot !== -1) {
    return filePath.substring(lastDot + 1).toLowerCase();
  }
  
  return '';
}

/**
 * Check if a file type is commonly used for audio processing
 * @param filePath The file path to check
 * @returns True if it's a common audio file type
 */
export function isAudioFile(filePath: string): boolean {
  const audioExtensions = [
    'wav', 'mp3', 'flac', 'aac', 'ogg', 'wma', 'm4a', 'aiff', 'au', 'ra'
  ];
  
  const extension = getFileExtension(filePath);
  return audioExtensions.includes(extension);
}

/**
 * Check if a file type is commonly used for video processing
 * @param filePath The file path to check
 * @returns True if it's a common video file type
 */
export function isVideoFile(filePath: string): boolean {
  const videoExtensions = [
    'mp4', 'avi', 'mkv', 'mov', 'wmv', 'flv', 'webm', 'm4v', '3gp', 'ogv'
  ];
  
  const extension = getFileExtension(filePath);
  return videoExtensions.includes(extension);
}

/**
 * Get a user-friendly description of a file type
 * @param filePath The file path
 * @returns Description of the file type
 */
export function getFileTypeDescription(filePath: string): string {
  const extension = getFileExtension(filePath);
  
  if (isAudioFile(filePath)) {
    return `Audio file (${extension.toUpperCase()})`;
  }
  
  if (isVideoFile(filePath)) {
    return `Video file (${extension.toUpperCase()})`;
  }
  
  const commonTypes: Record<string, string> = {
    'txt': 'Text file',
    'md': 'Markdown file',
    'json': 'JSON file',
    'xml': 'XML file',
    'csv': 'CSV file',
    'pdf': 'PDF document',
    'doc': 'Word document',
    'docx': 'Word document',
    'xls': 'Excel spreadsheet',
    'xlsx': 'Excel spreadsheet',
    'ppt': 'PowerPoint presentation',
    'pptx': 'PowerPoint presentation',
    'zip': 'ZIP archive',
    'tar': 'TAR archive',
    'gz': 'GZIP archive',
    'jpg': 'JPEG image',
    'jpeg': 'JPEG image',
    'png': 'PNG image',
    'gif': 'GIF image',
    'bmp': 'Bitmap image',
    'svg': 'SVG image',
  };
  
  return commonTypes[extension] || (extension ? `${extension.toUpperCase()} file` : 'Unknown file type');
}