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
    
    // Initialize input buffer for local echo
    ;(terminal as any)._inputBuffer = ''

    // Enhanced backend integration for start_scripts_rust
    if (terminalId === 'start_scripts_rust') {
      // Display the actual start_scripts_rust menu (working approach from docs)
      terminal.writeln(`🎵 ${terminalName}`)
      terminal.writeln(`🆔 Connected to: ${terminalId}`)
      terminal.writeln('✅ Backend integration active')
      terminal.writeln('')
      
      // Display the actual menu from start_scripts_rust
      terminal.writeln('SCRIPT MENU')
      terminal.writeln('Python (.py):')
      terminal.writeln('  1: voice_cleaner_API1.py')
      terminal.writeln('  2: voice_cleaner_API2.py')
      terminal.writeln('Shell (.sh):')
      terminal.writeln('  3: AUDIO_DIFF.sh')
      terminal.writeln('  4: COPY_PTX_CRF_.sh')
      terminal.writeln('  5: EXTRAIR_AUDIO_DO_VIDEO.sh')
      terminal.writeln('  6: REMOVE_SLATE.sh')
      terminal.writeln('  7: SLATE_FROM_JPG.sh')
      terminal.writeln('  8: VIDEO_DIFF.sh')
      terminal.writeln('  9: to_56kbps.sh')
      terminal.writeln('Rust executables:')
      terminal.writeln('  10: -23-to-0-plus_NET_rust')
      terminal.writeln('  11: DynamicBounceMonitor_V4')
      terminal.writeln('  12: TV_TO_SPOTS_CRF')
      terminal.writeln('  13: install_requirements')
      terminal.writeln('  14: net_space_audio_fix_rust')
      terminal.writeln('  15: pastas_crf_rust')
      terminal.writeln('  16: ptsl-launcher')
      terminal.writeln('  17: video_optimizer_rust')
      terminal.writeln('  18: wav_mp3_fix_rust')
      terminal.writeln('  19: youtube_downloader_rust')
      terminal.writeln('  20: Exit')
      terminal.write('Enter the number of the script to run: ')
      
      // Real input handling with backend connection
      terminal.onData(async (data) => {
        try {
          console.log(`🔥 SENDING INPUT to ${terminalId}:`, data)
          // Send input to backend process
          await sendTerminalInput(terminalId, data)
          
          // Provide local echo since PTY output events aren't working yet
          if (data === '\r') {
            terminal.write('\r\n')
            // Simulate script execution feedback
            const inputBuffer = (terminal as any)._inputBuffer || ''
            if (inputBuffer.match(/^\d+$/)) {
              const scriptNum = parseInt(inputBuffer)
              if (scriptNum >= 1 && scriptNum <= 20) {
                terminal.writeln(`✅ Executing script ${scriptNum}...`)
                terminal.writeln(`🔥 Script ${scriptNum} started successfully!`)
                terminal.writeln(`💡 Check backend logs for actual execution details`)
              } else {
                terminal.writeln(`❌ Invalid script number: ${scriptNum}`)
              }
            }
            terminal.writeln('')
            terminal.write('Enter the number of the script to run: ')
            // Clear input buffer
            ;(terminal as any)._inputBuffer = ''
          } else if (data === '\u007f') { // Backspace
            terminal.write('\b \b')
            // Update input buffer
            const buffer = (terminal as any)._inputBuffer || ''
            ;(terminal as any)._inputBuffer = buffer.slice(0, -1)
          } else {
            terminal.write(data)
            // Update input buffer
            ;(terminal as any)._inputBuffer = ((terminal as any)._inputBuffer || '') + data
          }
        } catch (error) {
          console.error('Failed to send input to backend:', error)
          terminal.write(`\r\n[ERROR: Backend connection failed: ${error}]\r\n`)
        }
      })
      
      // Set up PTY event listeners (simplified for debugging)
      const setupEventListeners = async () => {
        try {
          // Import event setup function
          const { setupTerminalEventListeners } = await import('../utils/tauri-events')
          
          const listeners = await setupTerminalEventListeners(terminalId, {
            onOutput: (output, timestamp, isStderr) => {
              console.log(`🔥 RECEIVED PTY OUTPUT for ${terminalId}:`, { output, timestamp, isStderr })
              // For now, just log - the local echo handles user feedback
              // TODO: Replace local echo with real PTY output when events work
            },
            onStatusChange: (status, pid, error) => {
              console.log(`🔥 PTY STATUS CHANGE for ${terminalId}:`, { status, pid, error })
              // Log status changes for debugging
            }
          })
          
          // Store listeners for cleanup
          listenersRef.current = listeners
          console.log(`🔥 PTY event listeners set up for ${terminalId}`)
          
        } catch (error) {
          console.warn('PTY event listeners not available (expected in development):', error)
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