import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import TabManager from '../TabManager'


// Mock the store hooks
const mockSetActiveTerminal = vi.fn()

vi.mock('../../store/app-store', () => ({
  useTerminalState: vi.fn(() => ({
    terminals: [
      {
        id: 'terminal-1',
        name: 'Super Fast Normalizer',
        launcher_executable: 'normalizer',
        working_directory: '/path/to/normalizer',
        environment_variables: {},
        auto_start: true
      },
      {
        id: 'terminal-2',
        name: 'Start Scripts',
        launcher_executable: 'scripts',
        working_directory: '/path/to/scripts',
        environment_variables: {},
        auto_start: true
      }
    ],
    activeTerminalId: 'terminal-1',
    processInfo: {
      'terminal-1': {
        terminal_id: 'terminal-1',
        status: 'Running',
        pid: 1234,
        started_at: '2024-01-01T12:00:00Z'
      },
      'terminal-2': {
        terminal_id: 'terminal-2',
        status: 'Idle',
        pid: 1235,
        started_at: '2024-01-01T12:00:01Z'
      }
    },
  })),
  useAppActions: vi.fn(() => ({
    setActiveTerminal: mockSetActiveTerminal,
  })),
}))

describe('TabManager', () => {

  it('should render terminals as tabs', () => {
    render(<TabManager />)
    
    expect(screen.getByRole('tab', { name: /Super Fast Normalizer/ })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: /Start Scripts/ })).toBeInTheDocument()
  })

  it('should show active tab correctly', () => {
    render(<TabManager />)
    
    const activeTab = screen.getByRole('tab', { selected: true })
    expect(activeTab).toHaveTextContent('Super Fast Normalizer')
  })

  it('should call setActiveTerminal when tab is clicked', () => {
    render(<TabManager />)
    
    const tab = screen.getByText('Start Scripts')
    fireEvent.click(tab)
    
    expect(mockSetActiveTerminal).toHaveBeenCalledWith('terminal-2')
  })

  it('should call onTabChange callback when tab is clicked', () => {
    const mockOnTabChange = vi.fn()
    render(<TabManager onTabChange={mockOnTabChange} />)
    
    const tab = screen.getByText('Start Scripts')
    fireEvent.click(tab)
    
    expect(mockOnTabChange).toHaveBeenCalledWith('terminal-2')
  })

  it('should show keyboard shortcuts help', () => {
    render(<TabManager />)
    
    expect(screen.getByText(/⌘1-5: Switch tabs/)).toBeInTheDocument()
  })
})