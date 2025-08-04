import { useEffect, useRef } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'

// MINIMAL TERMINAL - No CSS imports, no store, no Tauri
export default function MinimalTerminal({ terminalName }: { terminalName: string }) {
  const terminalRef = useRef<HTMLDivElement>(null)
  const xtermRef = useRef<Terminal | null>(null)

  useEffect(() => {
    if (!terminalRef.current || xtermRef.current) return

    try {
      console.log('🧪 Testing minimal terminal without CSS imports...')
      
      const xterm = new Terminal({
        cursorBlink: true,
        fontFamily: 'Monaco, Menlo, monospace',
        fontSize: 13,
        theme: {
          background: '#1e1e1e',
          foreground: '#d4d4d4'
        }
      })

      const fitAddon = new FitAddon()
      xterm.loadAddon(fitAddon)
      xterm.open(terminalRef.current)
      fitAddon.fit()

      xterm.writeln(`✅ ${terminalName} - Minimal Terminal`)
      xterm.writeln('No CSS imports, no store, no Tauri')
      xterm.writeln('')
      xterm.write('$ ')

      xtermRef.current = xterm
      console.log('✅ Minimal terminal without CSS imports successful')

    } catch (error) {
      console.error('❌ Minimal terminal failed:', error)
    }

    return () => {
      if (xtermRef.current) {
        xtermRef.current.dispose()
        xtermRef.current = null
      }
    }
  }, [terminalName])

  return (
    <div style={{ height: '100%', padding: '20px' }}>
      <h3>🧪 {terminalName} - Minimal Test</h3>
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