// TEST 1: Bare Import Test
// Goal: See if just importing xterm.js breaks the app (without using it)

import { Terminal } from '@xterm/xterm'

export default function Test1_BareImport() {
  // Reference Terminal to avoid TS error, but don't actually use it
  const terminalClass = Terminal
  
  return (
    <div style={{ 
      height: '100%', 
      display: 'flex', 
      alignItems: 'center', 
      justifyContent: 'center',
      flexDirection: 'column',
      gap: '20px'
    }}>
      <h2>🧪 Test 1: Bare Import</h2>
      <p>✅ If you see this, importing xterm.js doesn't break the app</p>
      <p>❌ If black screen, the import itself is the problem</p>
      <div style={{ 
        fontSize: '12px', 
        opacity: 0.7,
        textAlign: 'center'
      }}>
        <p>Terminal imported but not used</p>
        <p>This tests if the import statement itself causes the crash</p>
        <p>Terminal class available: {terminalClass ? 'Yes' : 'No'}</p>
      </div>
    </div>
  )
}