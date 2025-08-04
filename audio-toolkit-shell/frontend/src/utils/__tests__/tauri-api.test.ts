import { describe, it, expect, beforeEach, vi } from 'vitest'
import { 
  validateUserAccess,
  sendTerminalInput,
  getTerminalOutput,
  handleFileDrop,
  handleTauriError,
  safeTauriInvoke
} from '../tauri-api'

// Mock the Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const mockInvoke = vi.mocked((await import('@tauri-apps/api/core')).invoke)

describe('tauri-api', () => {
  beforeEach(() => {
    mockInvoke.mockReset()
  })

  describe('validateUserAccess', () => {
    it('should call validate_user_access command', async () => {
      const mockResponse = {
        auth_status: { is_authenticated: true, user_id: 'test-user', permissions: [] },
        version_status: 'Current',
        server_message: 'Welcome'
      }
      
      mockInvoke.mockResolvedValue(mockResponse)
      
      const result = await validateUserAccess()
      
      expect(mockInvoke).toHaveBeenCalledWith('validate_user_access')
      expect(result).toEqual(mockResponse)
    })

    it('should handle authentication errors', async () => {
      mockInvoke.mockRejectedValue(new Error('Authentication failed'))
      
      await expect(validateUserAccess()).rejects.toThrow('Authentication failed')
    })
  })

  describe('sendTerminalInput', () => {
    it('should send input to terminal', async () => {
      mockInvoke.mockResolvedValue(undefined)
      
      await sendTerminalInput('terminal-1', 'ls -la')
      
      expect(mockInvoke).toHaveBeenCalledWith('send_terminal_input', {
        terminal_id: 'terminal-1',
        input: 'ls -la'
      })
    })
  })

  describe('getTerminalOutput', () => {
    it('should get terminal output', async () => {
      const mockOutput = 'total 0\ndrwxr-xr-x  2 user  staff  64 Jan  1 12:00 .'
      mockInvoke.mockResolvedValue(mockOutput)
      
      const result = await getTerminalOutput('terminal-1')
      
      expect(mockInvoke).toHaveBeenCalledWith('get_terminal_output', {
        terminal_id: 'terminal-1'
      })
      expect(result).toBe(mockOutput)
    })
  })

  describe('handleFileDrop', () => {
    it('should handle file drop', async () => {
      mockInvoke.mockResolvedValue(undefined)
      
      const filePaths = ['/path/to/file1.wav', '/path/to/file2.mp3']
      await handleFileDrop('terminal-1', filePaths)
      
      expect(mockInvoke).toHaveBeenCalledWith('handle_file_drop', {
        terminal_id: 'terminal-1',
        file_paths: filePaths
      })
    })
  })

  describe('handleTauriError', () => {
    it('should handle string errors', () => {
      const result = handleTauriError('Test error message')
      expect(result).toBe('Test error message')
    })

    it('should handle Error objects', () => {
      const error = new Error('Test error')
      const result = handleTauriError(error)
      expect(result).toBe('Test error')
    })

    it('should handle unknown errors', () => {
      const result = handleTauriError({ unknown: 'error' })
      expect(result).toBe('An unknown error occurred')
    })
  })

  describe('safeTauriInvoke', () => {
    it('should return success result on successful invoke', async () => {
      const mockData = { test: 'data' }
      mockInvoke.mockResolvedValue(mockData)
      
      const result = await safeTauriInvoke('test_command', { arg: 'value' })
      
      expect(result).toEqual({ success: true, data: mockData })
      expect(mockInvoke).toHaveBeenCalledWith('test_command', { arg: 'value' })
    })

    it('should return error result on failed invoke', async () => {
      mockInvoke.mockRejectedValue(new Error('Command failed'))
      
      const result = await safeTauriInvoke('test_command')
      
      expect(result).toEqual({ success: false, error: 'Command failed' })
    })
  })
})