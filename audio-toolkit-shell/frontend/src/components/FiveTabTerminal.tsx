// FIVE TAB TERMINAL - Built on proven CleanTerminal foundation
// Clean architecture with no mixed component references

import { useState, useEffect } from 'react'

const TERMINAL_TABS = [
  { id: 'start-scripts', name: 'Start Scripts', shortcut: '⌘1' },
  { id: 'audio-normalizer', name: 'Audio Normalizer', shortcut: '⌘2' },
  { id: 'session-monitor', name: 'Session Monitor', shortcut: '⌘3' },
  { id: 'ptsl-launcher', name: 'Pro Tools Session Launcher', shortcut: '⌘4' },
  { id: 'fifth-launcher', name: 'Fifth Launcher', shortcut: '⌘5' }
]

export default function FiveTabTerminal() {
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
      
      {/* Terminal Content Area */}
      <div style={{ flex: 1, position: 'relative' }}>
        {TERMINAL_TABS.map((tab, index) => (
          <div
            key={tab.id}
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              display: activeTab === index ? 'block' : 'none'
            }}
          >
            {/* Each tab gets its own CleanTerminal instance */}
            {activeTab === index && (
              <div style={{ height: '100%' }}>
                {/* Terminal Header */}
                <div style={{
                  padding: '10px',
                  background: '#2d2d2d',
                  borderBottom: '1px solid #404040',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '10px'
                }}>
                  <span style={{ fontSize: '16px' }}>🎵</span>
                  <h3 style={{ margin: 0, fontSize: '16px' }}>{tab.name}</h3>
                  <span style={{ 
                    fontSize: '12px', 
                    opacity: 0.7,
                    background: '#404040',
                    padding: '2px 6px',
                    borderRadius: '3px'
                  }}>
                    Tab {index + 1} of 5
                  </span>
                </div>

                {/* CleanTerminal Instance */}
                <div style={{ height: 'calc(100% - 60px)' }}>
                  <CleanTerminalInstance 
                    tabName={tab.name}
                    tabId={tab.id}
                    tabIndex={index}
                  />
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}

// Import the clean terminal component (proven to work)
import CleanTerminal from './CleanTerminal'

// Individual CleanTerminal instance for each tab
function CleanTerminalInstance({ tabName, tabId, tabIndex }: { 
  tabName: string
  tabId: string 
  tabIndex: number 
}) {
  // Tab 1 (index 0) gets the actual start_scripts_rust terminal ID
  const actualTerminalId = tabIndex === 0 ? 'start_scripts_rust' : tabId
  
  return (
    <CleanTerminal 
      terminalId={actualTerminalId}
      terminalName={tabName}
    />
  )
}