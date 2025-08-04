// CLEAN TERMINAL - Enhanced with backend connectivity for start_scripts_rust
// Clean architecture with no mixed component references

import { useEffect, useRef } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { sendTerminalInput } from '../utils/tauri-api'

interface CleanTerminalProps {
  terminalId: string
  terminalName: string
}

export default function CleanTerminal({ terminalId, terminalName }: CleanTerminalProps) {
  const terminalRef = useRef<HTMLDivElement>(null)
  const xtermRef = useRef<Terminal | null>(null)
  const fitAddonRef = useRef<FitAddon | null>(null)
  const listenersRef = useRef<any>(null)

  useEffect(() => {
    if (!terminalRef.current || xtermRef.current) return

    // Create terminal with proven configuration
    const terminal = new Terminal({
      cursorBlink: true,
      cursorStyle: 'block',
      fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
      fontSize: 13,
      lineHeight: 1.2,
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
        cursor: '#ffffff'
      },
      allowProposedApi: true,
      allowTransparency: false,
      convertEol: true,
      scrollback: 10000,
      tabStopWidth: 4
    })

    // Add addons (proven to work)
    const fitAddon = new FitAddon()
    const webLinksAddon = new WebLinksAddon()

    terminal.loadAddon(fitAddon)
    terminal.loadAddon(webLinksAddon)

    // Open terminal
    terminal.open(terminalRef.current)
    fitAddon.fit()

    // Store references
    xtermRef.current = terminal
    fitAddonRef.current = fitAddon

    // Enhanced backend integration for start_scripts_rust
    if (terminalId === 'start_scripts_rust') {
      // Initialize terminal - wait for real PTY output
      terminal.writeln(`🎵 ${terminalName}`)
      terminal.writeln(`🆔 Connected to: ${terminalId}`)
      terminal.writeln('✅ PTY backend integration active')
      terminal.writeln('⏳ Waiting for executable output from PTY...')
      terminal.writeln('💡 The real menu should appear below when PTY events work')
      terminal.writeln('')
      
      // Real input handling with backend connection - NO LOCAL ECHO
      terminal.onData(async (data) => {
        try {
          console.log(`🔥 SENDING INPUT to ${terminalId}:`, data)
          // Send input to backend process - let PTY handle all output
          await sendTerminalInput(terminalId, data)
        } catch (error) {
          console.error('Failed to send input to backend:', error)
          terminal.write(`\r\n[ERROR: Backend connection failed: ${error}]\r\n`)
        }
      })
      
      // Set up PTY event listeners - CRITICAL for interactive scripts
      const setupEventListeners = async () => {
        try {
          // Import event setup function
          const { setupTerminalEventListeners } = await import('../utils/tauri-events')
          
          const listeners = await setupTerminalEventListeners(terminalId, {
            onOutput: (output, timestamp, isStderr) => {
              console.log(`🔥🔥🔥 RECEIVED PTY OUTPUT for ${terminalId}:`, { output, timestamp, isStderr })
              if (xtermRef.current) {
                if (isStderr) {
                  // Write stderr in red
                  xtermRef.current.write(`\x1b[31m${output}\x1b[0m`)
                } else {
                  // Write stdout - this includes menu, prompts, and all interactive output
                  xtermRef.current.write(output)
                }
              }
            },
            onStatusChange: (status, pid, error) => {
              console.log(`🔥🔥🔥 PTY STATUS CHANGE for ${terminalId}:`, { status, pid, error })
              if (error && xtermRef.current) {
                xtermRef.current.writeln(`\r\n[Process error: ${error}]\r\n`)
              }
            }
          })
          
          // Store listeners for cleanup
          listenersRef.current = listeners
          console.log(`🔥 PTY event listeners set up for ${terminalId} - waiting for output events`)
          
        } catch (error) {
          console.error('CRITICAL: PTY event listeners failed:', error)
          if (xtermRef.current) {
            xtermRef.current.writeln(`\r\n❌ CRITICAL: PTY events not working`)
            xtermRef.current.writeln(`❌ Interactive scripts (like script 19) won't work properly`)
            xtermRef.current.writeln(`💡 Check backend PTY plugin configuration\r\n`)
          }
        }
      }
      
      // Call async setup function
      setupEventListeners()
      
    } else {
      // Mock terminal for other tabs (will be enhanced later)
      terminal.writeln(`🎵 ${terminalName}`)
      terminal.writeln(`🆔 Terminal ID: ${terminalId}`)
      terminal.writeln('✅ Clean terminal - no black screen')
      terminal.writeln('✅ Ready for backend integration...')
      terminal.write('$ ')

      // Basic mock input handling for other tabs
      terminal.onData((data) => {
        if (data === '\r') {
          terminal.write('\r\n$ ')
        } else if (data === '\u007f') { // Backspace
          terminal.write('\b \b')
        } else {
          terminal.write(data)
        }
      })
    }

    // Handle window resize
    const handleResize = () => {
      if (fitAddon) {
        fitAddon.fit()
      }
    }
    window.addEventListener('resize', handleResize)

    return () => {
      window.removeEventListener('resize', handleResize)
      
      // Cleanup event listeners
      if (listenersRef.current) {
        try {
          import('../utils/tauri-events').then(({ cleanupEventListeners }) => {
            cleanupEventListeners(listenersRef.current)
          })
        } catch (error) {
          console.warn('Failed to cleanup event listeners:', error)
        }
        listenersRef.current = null
      }
      
      if (xtermRef.current) {
        xtermRef.current.dispose()
        xtermRef.current = null
      }
      fitAddonRef.current = null
    }
  }, [terminalId, terminalName])

  return (
    <div style={{ height: '100%', padding: '10px' }}>
      <div 
        ref={terminalRef}
        style={{ 
          height: '100%',
          border: '1px solid #404040',
          borderRadius: '4px'
        }}
      />
    </div>
  )
}