// PTY Debug Terminal - Minimal test for PTY events
import { useEffect, useRef } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { sendTerminalInput } from '../utils/tauri-api'
import { setupTerminalEventListeners } from '../utils/tauri-events'

interface PTYDebugTerminalProps {
  terminalId: string
}

export default function PTYDebugTerminal({ terminalId }: PTYDebugTerminalProps) {
  const terminalRef = useRef<HTMLDivElement>(null)
  const xtermRef = useRef<Terminal | null>(null)
  const listenersRef = useRef<any>(null)

  useEffect(() => {
    if (!terminalRef.current || xtermRef.current) return

    console.log(`🔥 PTY DEBUG: Setting up terminal for ${terminalId}`)

    // Create minimal terminal
    const terminal = new Terminal({
      cursorBlink: true,
      fontFamily: 'Monaco, Menlo, monospace',
      fontSize: 13,
      theme: { background: '#1e1e1e', foreground: '#d4d4d4' }
    })

    const fitAddon = new FitAddon()
    terminal.loadAddon(fitAddon)
    terminal.open(terminalRef.current)
    fitAddon.fit()

    xtermRef.current = terminal

    // Debug info
    terminal.writeln(`🔥 PTY DEBUG TERMINAL`)
    terminal.writeln(`🆔 Terminal ID: ${terminalId}`)
    terminal.writeln(`⏳ Setting up PTY event listeners...`)

    // Set up PTY event listeners
    const setupEvents = async () => {
      try {
        const listeners = await setupTerminalEventListeners(terminalId, {
          onOutput: (output, timestamp, isStderr) => {
            console.log(`🔥🔥🔥 PTY OUTPUT RECEIVED:`, { terminalId, output, timestamp, isStderr })
            if (xtermRef.current) {
              if (isStderr) {
                xtermRef.current.write(`\x1b[31m${output}\x1b[0m`)
              } else {
                xtermRef.current.write(output)
              }
            }
          },
          onStatusChange: (status, pid, error) => {
            console.log(`🔥🔥🔥 PTY STATUS CHANGE:`, { terminalId, status, pid, error })
            if (xtermRef.current) {
              xtermRef.current.writeln(`\r\n[Status: ${status}${pid ? ` PID: ${pid}` : ''}${error ? ` Error: ${error}` : ''}]\r\n`)
            }
          }
        })

        listenersRef.current = listeners
        terminal.writeln(`✅ PTY event listeners connected!`)
        terminal.writeln(`💡 Type commands to test PTY communication`)
        terminal.write(`$ `)

      } catch (error) {
        console.error(`🔥 PTY EVENT SETUP FAILED:`, error)
        terminal.writeln(`❌ PTY event setup failed: ${error}`)
      }
    }

    setupEvents()

    // Handle input
    terminal.onData(async (data) => {
      try {
        console.log(`🔥 SENDING TO PTY:`, { terminalId, data })
        await sendTerminalInput(terminalId, data)
      } catch (error) {
        console.error(`🔥 PTY INPUT FAILED:`, error)
        terminal.write(`\r\n[ERROR: ${error}]\r\n$ `)
      }
    })

    return () => {
      if (listenersRef.current) {
        import('../utils/tauri-events').then(({ cleanupEventListeners }) => {
          cleanupEventListeners(listenersRef.current)
        })
      }
      if (xtermRef.current) {
        xtermRef.current.dispose()
      }
    }
  }, [terminalId])

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