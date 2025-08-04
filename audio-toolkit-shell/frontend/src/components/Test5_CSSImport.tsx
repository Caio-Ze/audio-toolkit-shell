// TEST 5: CSS Import Test
// Goal: See if importing xterm.js CSS breaks the app

import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
// THIS IS THE SUSPECTED CULPRIT:
import '@xterm/xterm/css/xterm.css'
import { useState, useEffect, useRef } from 'react'

export default function Test5_CSSImport() {
  const [status, setStatus] = useState<'creating' | 'success' | 'error'>('creating')
  const [error, setError] = useState<string | null>(null)
  const terminalRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    let terminal: Terminal | null = null

    const runTest = async () => {
      try {
        console.log('🧪 Test 5: Testing with CSS import...')
        setStatus('creating')
        
        // Create full terminal with CSS
        terminal = new Terminal({
          cursorBlink: true,
          fontFamily: 'Monaco, Menlo, monospace',
          fontSize: 13,
          theme: {
            background: '#1e1e1e',
            foreground: '#d4d4d4'
          }
        })

        if (!terminalRef.current) {
          throw new Error('Terminal ref not available')
        }

        // Load addons
        const fitAddon = new FitAddon()
        const webLinksAddon = new WebLinksAddon()
        
        terminal.loadAddon(fitAddon)
        terminal.loadAddon(webLinksAddon)

        // Open terminal
        terminal.open(terminalRef.current)
        fitAddon.fit()

        // Write test content
        terminal.writeln('✅ CSS Import Test')
        terminal.writeln('✅ xterm.css imported successfully!')
        terminal.writeln('✅ Terminal styling should look proper')
        terminal.writeln('')
        terminal.writeln('If you see this with proper styling,')
        terminal.writeln('CSS import is NOT the problem!')
        terminal.write('$ ')

        setStatus('success')

      } catch (err) {
        console.error('❌ CSS import test failed:', err)
        setError(err instanceof Error ? err.message : 'Unknown error')
        setStatus('error')
      }
    }

    runTest()

    return () => {
      if (terminal) {
        terminal.dispose()
      }
    }
  }, [])

  const getStatusMessage = () => {
    switch (status) {
      case 'creating':
        return { text: '⏳ Testing with CSS import...', color: '#ffaa00' }
      case 'success':
        return { text: '✅ CSS import works!', color: '#0dbc79' }
      case 'error':
        return { text: '❌ CSS import failed!', color: '#ff6b6b' }
    }
  }

  const statusMessage = getStatusMessage()

  return (
    <div style={{ 
      height: '100%', 
      display: 'flex',
      flexDirection: 'column',
      gap: '20px',
      padding: '20px'
    }}>
      <div style={{ textAlign: 'center' }}>
        <h2>🧪 Test 5: CSS Import</h2>
        <p style={{ color: statusMessage.color }}>{statusMessage.text}</p>
        
        {error && (
          <p style={{ color: '#ff6b6b', fontSize: '12px' }}>Error: {error}</p>
        )}
        
        <div style={{ 
          fontSize: '12px', 
          opacity: 0.7,
          marginTop: '10px'
        }}>
          <p>Testing: import '@xterm/xterm/css/xterm.css'</p>
          <p>This is the prime suspect for the black screen</p>
        </div>
      </div>

      {/* Terminal Container */}
      <div style={{ flex: 1, border: '1px solid #404040', borderRadius: '4px' }}>
        <div 
          ref={terminalRef}
          style={{ 
            height: '100%',
            padding: '8px'
          }}
        />
      </div>
    </div>
  )
}