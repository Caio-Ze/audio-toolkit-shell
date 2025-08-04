// UNIFIED SOLUTION - Safe 5-tab interface using CleanTerminal foundation (BLACK_SCREEN_PROBLEM.md)
import { useState, useEffect } from 'react'
import CleanTerminal from './components/CleanTerminal'

const TERMINAL_TABS = [
  { id: 'start_scripts_rust', name: 'Start Scripts', shortcut: '⌘1' },
  { id: 'audio_normalizer', name: 'Audio Normalizer', shortcut: '⌘2' },
  { id: 'session_monitor', name: 'Session Monitor', shortcut: '⌘3' },
  { id: 'ptsl_launcher', name: 'Pro Tools Session Launcher', shortcut: '⌘4' },
  { id: 'fifth_launcher', name: 'Fifth Launcher', shortcut: '⌘5' }
]

function App() {
  const [activeTab, setActiveTab] = useState(0)

  // Handle keyboard shortcuts ⌘1-5
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.metaKey || event.ctrlKey) {
        const keyNumber = parseInt(event.key)
        if (keyNumber >= 1 && keyNumber <= 5) {
          event.preventDefault()
          setActiveTab(keyNumber - 1)
          console.log(`🎵 Switched to tab ${keyNumber}: ${TERMINAL_TABS[keyNumber - 1].name}`)
        }
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [])

  const activeTerminal = TERMINAL_TABS[activeTab]

  return (
    <div style={{
      height: '100vh',
      width: '100vw',
      display: 'flex',
      flexDirection: 'column',
      background: '#1e1e1e',
      color: '#d4d4d4'
    }}>
      {/* Tab Bar */}
      <div style={{
        display: 'flex',
        background: '#2d2d2d',
        borderBottom: '1px solid #404040',
        minHeight: '48px',
        alignItems: 'center'
      }}>
        {TERMINAL_TABS.map((tab, index) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(index)}
            style={{
              padding: '12px 16px',
              background: activeTab === index ? '#404040' : 'transparent',
              border: 'none',
              color: activeTab === index ? '#ffffff' : '#cccccc',
              cursor: 'pointer',
              fontSize: '14px',
              borderRadius: activeTab === index ? '4px 4px 0 0' : '0',
              transition: 'all 0.2s ease',
              display: 'flex',
              alignItems: 'center',
              gap: '8px'
            }}
            onMouseEnter={(e) => {
              if (activeTab !== index) {
                e.currentTarget.style.background = '#353535'
              }
            }}
            onMouseLeave={(e) => {
              if (activeTab !== index) {
                e.currentTarget.style.background = 'transparent'
              }
            }}
          >
            <span>{tab.name}</span>
            <span style={{ 
              fontSize: '12px', 
              opacity: 0.7 
            }}>
              {tab.shortcut}
            </span>
          </button>
        ))}
      </div>
      
      {/* Content Area - Single CleanTerminal based on active tab */}
      <div style={{ flex: 1 }}>
        <CleanTerminal 
          terminalId={activeTerminal.id}
          terminalName={activeTerminal.name}
        />
      </div>
    </div>
  )
}

export default App