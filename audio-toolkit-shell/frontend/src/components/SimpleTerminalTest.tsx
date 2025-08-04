import { useEffect, useRef } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'

// MINIMAL TERMINAL TEST - Just xterm.js without Tauri dependencies
export default function SimpleTerminalTest() {
  const terminalRef = useRef<HTMLDivElement>(null)
  const xtermRef = useRef<Terminal | null>(null)

  useEffect(() => {
    if (!terminalRef.current || xtermRef.current) return

    try {
      console.log('🧪 Testing minimal xterm.js initialization...')
      
      // Create basic terminal
      const xterm = new Terminal({
        cursorBlink: true,
        fontFamily: 'Monaco, Menlo, monospace',
        fontSize: 13,
        theme: {
          background: '#1e1e1e',
          foreground: '#d4d4d4'
        }
      })

      // Add fit addon
      const fitAddon = new FitAddon()
      xterm.loadAddon(fitAddon)

      // Open terminal
      xterm.open(terminalRef.current)
      fitAddon.fit()

      // Write test message
      xterm.writeln('✅ Minimal terminal test successful!')
      xterm.writeln('If you see this, xterm.js works fine.')
      xterm.writeln('')
      xterm.write('$ ')

      xtermRef.current = xterm

      console.log('✅ Minimal terminal initialized successfully')

    } catch (error) {
      console.error('❌ Minimal terminal test failed:', error)
    }

    return () => {
      if (xtermRef.current) {
        xtermRef.current.dispose()
        xtermRef.current = null
      }
    }
  }, [])

  return (
    <div style={{ height: '100%', padding: '20px' }}>
      <h3>🧪 Minimal Terminal Test</h3>
      <div 
        ref={terminalRef}
        style={{ 
          height: 'calc(100% - 60px)',
          border: '1px solid #404040',
          borderRadius: '4px'
        }}
      />
    </div>
  )
}