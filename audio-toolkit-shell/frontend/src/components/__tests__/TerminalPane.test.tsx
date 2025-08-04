import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import TerminalPane from '../TerminalPane'

// Mock xterm.js
const mockTerminal = {
  loadAddon: vi.fn(),
  open: vi.fn(),
  onData: vi.fn(),
  onResize: vi.fn(),
  write: vi.fn(),
  writeln: vi.fn(),
  clear: vi.fn(),
  focus: vi.fn(),
  dispose: vi.fn(),
}

const mockFitAddon = {
  fit: vi.fn(),
}

vi.mock('@xterm/xterm', () => ({
  Terminal: vi.fn(() => mockTerminal),
}))

vi.mock('@xterm/addon-fit', () => ({
  FitAddon: vi.fn(() => mockFitAddon),
}))

vi.mock('@xterm/addon-web-links', () => ({
  WebLinksAddon: vi.fn(() => ({})),
}))

// Mock Tauri API
vi.mock('../../utils/tauri-api', () => ({
  sendTerminalInput: vi.fn(),
  getTerminalOutput: vi.fn(),
}))

// Mock Tauri events
vi.mock('../../utils/tauri-events', () => ({
  setupTerminalEventListeners: vi.fn(() => Promise.resolve({
    unlistenOutput: vi.fn(),
    unlistenStatus: vi.fn(),
  })),
  cleanupEventListeners: vi.fn(),
}))

// Mock file drop
vi.mock('../../utils/file-drop', () => ({
  setupDragAndDrop: vi.fn(() => vi.fn()),
}))

// Mock store
const mockUpdateProcessStatus = vi.fn()

vi.mock('../../store/app-store', () => ({
  useTerminalState: vi.fn(() => ({
    terminals: [
      {
        id: 'terminal-1',
        name: 'Test Terminal',
        launcher_executable: 'test',
        working_directory: '/test',
        environment_variables: {},
        auto_start: true,
      }
    ],
    processInfo: {
      'terminal-1': {
        terminal_id: 'terminal-1',
        status: 'Running',
        pid: 1234,
      }
    },
  })),
  useAppActions: vi.fn(() => ({
    updateProcessStatus: mockUpdateProcessStatus,
  })),
}))

// Mock ResizeObserver
globalThis.ResizeObserver = vi.fn(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

describe('TerminalPane', () => {

  it('should render terminal pane with header', () => {
    render(
      <TerminalPane 
        terminalId="terminal-1" 
        isActive={true} 
      />
    )

    expect(screen.getByText('Test Terminal')).toBeInTheDocument()
    expect(screen.getByText('Running')).toBeInTheDocument()
  })

  it('should show active state when isActive is true', () => {
    render(
      <TerminalPane 
        terminalId="terminal-1" 
        isActive={true} 
      />
    )

    const pane = screen.getByRole('tabpanel')
    expect(pane).toHaveClass('active')
  })

  it('should show inactive state when isActive is false', () => {
    render(
      <TerminalPane 
        terminalId="terminal-1" 
        isActive={false} 
      />
    )

    const pane = screen.getByRole('tabpanel')
    expect(pane).toHaveClass('inactive')
  })
})