// TEST 5: NoAddon Terminal Test
// Goal: Test the NoAddonTerminalPane component in isolation

import { useState, useEffect } from 'react'
import NoAddonTerminalPane from './NoAddonTerminalPane'
import { useAppStore } from '../store/app-store'

export default function Test5_NoAddonTerminal() {
  const [storeInitialized, setStoreInitialized] = useState(false)
  const { terminals, initialize } = useAppStore()

  useEffect(() => {
    const initStore = async () => {
      try {
        console.log('🧪 Test 5: Initializing store for NoAddon test...')
        await initialize()
        setStoreInitialized(true)
        console.log('✅ Store initialized, terminals:', terminals.length)
      } catch (error) {
        console.error('❌ Store initialization failed:', error)
      }
    }
    initStore()
  }, [initialize])

  if (!storeInitialized) {
    return (
      <div style={{ 
        height: '100%', 
        display: 'flex', 
        alignItems: 'center', 
        justifyContent: 'center',
        flexDirection: 'column',
        gap: '20px'
      }}>
        <h2>🧪 Test 5: NoAddon Terminal</h2>
        <p style={{ color: '#ffaa00' }}>⏳ Initializing store...</p>
      </div>
    )
  }

  if (terminals.length === 0) {
    return (
      <div style={{ 
        height: '100%', 
        display: 'flex', 
        alignItems: 'center', 
        justifyContent: 'center',
        flexDirection: 'column',
        gap: '20px'
      }}>
        <h2>🧪 Test 5: NoAddon Terminal</h2>
        <p style={{ color: '#ff6b6b' }}>❌ No terminals found</p>
        <p style={{ fontSize: '12px', opacity: 0.7 }}>
          Store initialized but no terminals available
        </p>
      </div>
    )
  }

  // Use the first terminal for testing
  const testTerminal = terminals[0]

  return (
    <div style={{ 
      height: '100%', 
      display: 'flex',
      flexDirection: 'column',
      gap: '10px',
      padding: '10px'
    }}>
      <div style={{ textAlign: 'center' }}>
        <h2>🧪 Test 5: NoAddon Terminal</h2>
        <p style={{ color: '#0dbc79' }}>✅ Testing NoAddonTerminalPane component</p>
        <div style={{ 
          fontSize: '12px', 
          opacity: 0.7,
          marginTop: '5px'
        }}>
          <p>Terminal: {testTerminal.name} (ID: {testTerminal.id})</p>
          <p>If you see this + working terminal below = SUCCESS!</p>
        </div>
      </div>

      {/* NoAddon Terminal Test */}
      <div style={{ flex: 1 }}>
        <NoAddonTerminalPane
          terminalId={testTerminal.id}
          isActive={true}
        />
      </div>
    </div>
  )
}