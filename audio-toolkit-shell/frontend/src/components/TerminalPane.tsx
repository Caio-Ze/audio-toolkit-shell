import { useEffect, useRef, useCallback, useState } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { useTerminalState, useAppActions } from '../store/app-store'
import { sendTerminalInput } from '../utils/tauri-api'
import { setupTerminalEventListeners, cleanupEventListeners } from '../utils/tauri-events'
import { setupDragAndDrop } from '../utils/file-drop'
import type { UnlistenFn } from '@tauri-apps/api/event'
import './TerminalPane.css'

interface TerminalPaneProps {
  terminalId: string
  isActive: boolean
  onResize?: (cols: number, rows: number) => void
}

export default function TerminalPane({ terminalId, isActive, onResize }: TerminalPaneProps) {
  const terminalRef = useRef<HTMLDivElement>(null)
  const xtermRef = useRef<Terminal | null>(null)
  const fitAddonRef = useRef<FitAddon | null>(null)
  const eventListenersRef = useRef<{ unlistenOutput?: UnlistenFn; unlistenStatus?: UnlistenFn } | null>(null)
  const dragCleanupRef = useRef<(() => void) | null>(null)
  const resizeObserverRef = useRef<ResizeObserver | null>(null)
  
  const [isInitialized, setIsInitialized] = useState(false)
  const [error, setError] = useState<string | null>(null)
  
  const { terminals, processInfo } = useTerminalState()
  const { updateProcessStatus } = useAppActions()
  
  const terminal = terminals.find(t => t.id === terminalId)
  const processStatus = processInfo[terminalId]

  // Initialize xterm.js terminal
  const initializeTerminal = useCallback(async () => {
    if (!terminalRef.current || xtermRef.current) return

    try {
      // Create terminal instance
      const xterm = new Terminal({
        cursorBlink: true,
        cursorStyle: 'block',
        fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
        fontSize: 13,
        lineHeight: 1.2,
        theme: {
          background: '#1e1e1e',
          foreground: '#d4d4d4',
          cursor: '#ffffff',
          cursorAccent: '#000000',
          selectionBackground: '#264f78',
          black: '#000000',
          red: '#cd3131',
          green: '#0dbc79',
          yellow: '#e5e510',
          blue: '#2472c8',
          magenta: '#bc3fbc',
          cyan: '#11a8cd',
          white: '#e5e5e5',
          brightBlack: '#666666',
          brightRed: '#f14c4c',
          brightGreen: '#23d18b',
          brightYellow: '#f5f543',
          brightBlue: '#3b8eea',
          brightMagenta: '#d670d6',
          brightCyan: '#29b8db',
          brightWhite: '#e5e5e5'
        },
        allowProposedApi: true,
        allowTransparency: false,
        convertEol: true,
        scrollback: 10000,
        tabStopWidth: 4
      })

      // Create and load addons
      const fitAddon = new FitAddon()
      const webLinksAddon = new WebLinksAddon()
      
      xterm.loadAddon(fitAddon)
      xterm.loadAddon(webLinksAddon)
      
      // Open terminal in DOM
      xterm.open(terminalRef.current)
      
      // Store references
      xtermRef.current = xterm
      fitAddonRef.current = fitAddon
      
      // Fit terminal to container
      fitAddon.fit()
      
      // Set up input handling - connect to backend processes
      xterm.onData(async (data) => {
        try {
          console.log(`Terminal ${terminalId} sending input to backend:`, data)
          
          // Send input directly to backend process (no local echo)
          await sendTerminalInput(terminalId, data)
        } catch (error) {
          console.error('Failed to send input to backend:', error)
          // Show error in terminal
          xterm.write(`\r\n[ERROR: Backend connection failed: ${error}]\r\n`)
        }
      })
      
      // Set up resize handling
      xterm.onResize(({ cols, rows }) => {
        onResize?.(cols, rows)
      })
      
      // Set up event listeners for terminal output (development mode - mock)
      try {
        const listeners = await setupTerminalEventListeners(terminalId, {
          onOutput: (output, _timestamp, isStderr) => {
            if (xtermRef.current) {
              if (isStderr) {
                // Write stderr in red
                xtermRef.current.write(`\x1b[31m${output}\x1b[0m`)
              } else {
                xtermRef.current.write(output)
              }
            }
          },
          onStatusChange: (status, _pid, error) => {
            updateProcessStatus(terminalId, status as any)
            if (error) {
              setError(`Process error: ${error}`)
            }
          }
        })
        
        eventListenersRef.current = listeners
      } catch (error) {
        console.warn('Failed to setup event listeners (development mode):', error)
        // Continue without event listeners in development
      }
      
      // Set up drag and drop
      if (terminalRef.current) {
        const cleanup = setupDragAndDrop(
          terminalRef.current,
          async (filePaths) => {
            try {
              // Insert file paths at cursor position
              const pathString = filePaths.map(path => `"${path}"`).join(' ')
              await sendTerminalInput(terminalId, pathString)
            } catch (error) {
              console.error('Failed to handle file drop:', error)
              setError('Failed to process dropped files')
            }
          }
        )
        dragCleanupRef.current = cleanup
      }
      
      // Set up resize observer
      if (terminalRef.current) {
        const resizeObserver = new ResizeObserver(() => {
          if (fitAddonRef.current && isActive) {
            fitAddonRef.current.fit()
          }
        })
        resizeObserver.observe(terminalRef.current)
        resizeObserverRef.current = resizeObserver
      }
      
      setIsInitialized(true)
      setError(null)
      
      // Connect to backend process output
      xterm.writeln(`\x1b[32m✓ ${terminal?.name || 'Terminal'} - Connecting to backend...\x1b[0m`)
      xterm.writeln('')
      
    } catch (error) {
      console.error('Failed to initialize terminal:', error)
      setError('Failed to initialize terminal')
    }
  }, [terminalId, terminal?.name, isActive, onResize, updateProcessStatus])

  // Cleanup function
  const cleanup = useCallback(() => {
    if (eventListenersRef.current) {
      cleanupEventListeners(eventListenersRef.current)
      eventListenersRef.current = null
    }
    
    if (dragCleanupRef.current) {
      dragCleanupRef.current()
      dragCleanupRef.current = null
    }
    
    if (resizeObserverRef.current) {
      resizeObserverRef.current.disconnect()
      resizeObserverRef.current = null
    }
    
    if (xtermRef.current) {
      xtermRef.current.dispose()
      xtermRef.current = null
    }
    
    fitAddonRef.current = null
    setIsInitialized(false)
  }, [])

  // Initialize terminal when component mounts or terminalId changes
  useEffect(() => {
    initializeTerminal()
    return cleanup
  }, [initializeTerminal, cleanup])

  // Handle active state changes
  useEffect(() => {
    if (isActive && fitAddonRef.current && xtermRef.current) {
      // Fit terminal when it becomes active
      setTimeout(() => {
        fitAddonRef.current?.fit()
        xtermRef.current?.focus()
      }, 0)
    }
  }, [isActive])

  // Handle window resize
  useEffect(() => {
    const handleResize = () => {
      if (isActive && fitAddonRef.current) {
        fitAddonRef.current.fit()
      }
    }

    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [isActive])

  // Clear error after some time
  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => setError(null), 5000)
      return () => clearTimeout(timer)
    }
  }, [error])

  if (!terminal) {
    return (
      <div className="terminal-pane terminal-pane-error">
        <div className="terminal-error">
          <h3>Terminal Not Found</h3>
          <p>Terminal with ID "{terminalId}" could not be found.</p>
        </div>
      </div>
    )
  }

  return (
    <div 
      className={`terminal-pane ${isActive ? 'active' : 'inactive'}`}
      id={`terminal-panel-${terminalId}`}
      role="tabpanel"
      aria-labelledby={`tab-${terminalId}`}
      tabIndex={isActive ? 0 : -1}
    >
      {error && (
        <div className="terminal-error-banner">
          <span className="error-icon">⚠️</span>
          <span className="error-message">{error}</span>
          <button 
            className="error-dismiss"
            onClick={() => setError(null)}
            aria-label="Dismiss error"
          >
            ×
          </button>
        </div>
      )}
      
      <div className="terminal-header">
        <div className="terminal-info">
          <span className="terminal-name">{terminal.name}</span>
          <span className="terminal-status">
            {processStatus ? (
              typeof processStatus.status === 'string' ? 
                processStatus.status : 
                `Error: ${processStatus.status.Error}`
            ) : 'Initializing...'}
          </span>
        </div>
        
        <div className="terminal-controls">
          <button
            className="terminal-control-btn"
            onClick={() => {
              if (xtermRef.current) {
                xtermRef.current.clear()
              }
            }}
            title="Clear terminal"
            aria-label="Clear terminal"
          >
            🗑️
          </button>
          
          <button
            className="terminal-control-btn"
            onClick={() => {
              if (fitAddonRef.current) {
                fitAddonRef.current.fit()
              }
            }}
            title="Fit terminal to window"
            aria-label="Fit terminal to window"
          >
            📐
          </button>
        </div>
      </div>
      
      <div 
        ref={terminalRef}
        className="terminal-container"
        style={{ 
          display: isActive ? 'block' : 'none',
          height: 'calc(100% - 40px)' // Account for header
        }}
      />
      
      {!isInitialized && !error && (
        <div className="terminal-loading">
          <div className="loading-spinner"></div>
          <span>Initializing terminal...</span>
        </div>
      )}
    </div>
  )
}