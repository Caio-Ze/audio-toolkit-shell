import { useEffect, useRef, useState } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'

function TerminalApp() {
  const terminalRef = useRef<HTMLDivElement>(null)
  const xtermRef = useRef<Terminal | null>(null)
  const fitAddonRef = useRef<FitAddon | null>(null)
  const [isConnected, setIsConnected] = useState(false)

  useEffect(() => {
    if (!terminalRef.current) return

    // Create xterm.js terminal instance
    const terminal = new Terminal({
      cursorBlink: true,
      cursorStyle: 'block',
      fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
      fontSize: 14,
      lineHeight: 1.2,
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
        cursor: '#ffffff',
        black: '#000000',
        red: '#cd3131',
        green: '#0dbc79',
        yellow: '#e5e510',
        blue: '#2472c8',
        magenta: '#bc3fbc',
        cyan: '#11a8cd',
        white: '#e5e5e5',
      },
      scrollback: 10000,
    })

    const fitAddon = new FitAddon()
    terminal.loadAddon(fitAddon)
    
    terminal.open(terminalRef.current)
    fitAddon.fit()

    xtermRef.current = terminal
    fitAddonRef.current = fitAddon

    // Connect to the backend process
    connectToBackend()

    return () => {
      terminal.dispose()
    }
  }, [])

  const connectToBackend = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      
      if (xtermRef.current) {
        xtermRef.current.writeln('🔧 WRAPPER EXECUTION TEST')
        xtermRef.current.writeln('═══════════════════════════════════════════════')
        xtermRef.current.writeln('')
        xtermRef.current.writeln('Testing: wrapper.sh → start_scripts_rust')
        xtermRef.current.writeln('Running test...')
        xtermRef.current.writeln('')
      }
      
      const output = await invoke('test_wrapper_execution') as string
      
      if (xtermRef.current) {
        xtermRef.current.write(output)
        setIsConnected(true)
      }
    } catch (error) {
      if (xtermRef.current) {
        xtermRef.current.writeln(`❌ Test failed: ${error}`)
      }
    }
  }

  const handleResize = () => {
    if (fitAddonRef.current) {
      fitAddonRef.current.fit()
    }
  }

  useEffect(() => {
    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [])

  return (
    <div style={{
      height: '100vh',
      display: 'flex',
      flexDirection: 'column',
      backgroundColor: '#1e1e1e'
    }}>
      <div style={{
        padding: '10px 20px',
        backgroundColor: '#2d2d2d',
        borderBottom: '1px solid #404040',
        color: '#d4d4d4',
        fontFamily: 'Monaco, monospace'
      }}>
        <h1 style={{ margin: 0, fontSize: '18px' }}>🎵 Audio Toolkit Shell</h1>
        <div style={{ fontSize: '12px', opacity: 0.7 }}>
          {isConnected ? '✅ Connected to start_scripts_rust' : '🔄 Connecting...'}
        </div>
      </div>
      
      <div 
        ref={terminalRef}
        style={{
          flex: 1,
          padding: '10px'
        }}
      />
    </div>
  )
}

export default TerminalApp