import { useEffect, useCallback } from 'react'
import { useTerminalState, useAppActions } from '../store/app-store'
import './TabManager.css'

interface TabManagerProps {
  onTabChange?: (terminalId: string) => void
}

export default function TabManager({ onTabChange }: TabManagerProps) {
  const { terminals, activeTerminalId, processInfo } = useTerminalState()
  const { setActiveTerminal } = useAppActions()

  const handleTabClick = useCallback((terminalId: string) => {
    setActiveTerminal(terminalId)
    onTabChange?.(terminalId)
  }, [setActiveTerminal, onTabChange])

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    console.log('🎹 TabManager: Key pressed:', {
      key: event.key,
      metaKey: event.metaKey,
      ctrlKey: event.ctrlKey,
      shiftKey: event.shiftKey
    })
    
    // Handle keyboard shortcuts for tab switching
    if (event.metaKey || event.ctrlKey) {
      const keyNumber = parseInt(event.key)
      console.log('🔢 TabManager: Parsed key number:', keyNumber)
      
      if (keyNumber >= 1 && keyNumber <= 5 && terminals[keyNumber - 1]) {
        console.log('✅ TabManager: Switching to tab', keyNumber, terminals[keyNumber - 1].name)
        event.preventDefault()
        handleTabClick(terminals[keyNumber - 1].id)
      }
      
      // Handle Cmd/Ctrl + Tab for next tab
      if (event.key === 'Tab' && !event.shiftKey) {
        console.log('➡️ TabManager: Next tab')
        event.preventDefault()
        const currentIndex = terminals.findIndex(t => t.id === activeTerminalId)
        const nextIndex = (currentIndex + 1) % terminals.length
        if (terminals[nextIndex]) {
          handleTabClick(terminals[nextIndex].id)
        }
      }
      
      // Handle Cmd/Ctrl + Shift + Tab for previous tab
      if (event.key === 'Tab' && event.shiftKey) {
        console.log('⬅️ TabManager: Previous tab')
        event.preventDefault()
        const currentIndex = terminals.findIndex(t => t.id === activeTerminalId)
        const prevIndex = currentIndex === 0 ? terminals.length - 1 : currentIndex - 1
        if (terminals[prevIndex]) {
          handleTabClick(terminals[prevIndex].id)
        }
      }
    }
  }, [terminals, activeTerminalId, handleTabClick])

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [handleKeyDown])

  const getStatusIndicator = (terminalId: string): string => {
    const info = processInfo[terminalId]
    if (!info) return 'inactive'
    
    const status = info.status
    if (typeof status === 'string') {
      switch (status) {
        case 'Starting':
          return 'starting'
        case 'Running':
          return 'running'
        case 'Idle':
          return 'idle'
        case 'Processing':
          return 'processing'
        case 'Terminated':
          return 'terminated'
        default:
          return 'inactive'
      }
    } else if (typeof status === 'object' && 'Error' in status) {
      return 'error'
    }
    
    return 'inactive'
  }

  const getStatusColor = (status: string): string => {
    switch (status) {
      case 'starting':
        return '#ffa500' // Orange
      case 'running':
        return '#00ff00' // Green
      case 'idle':
        return '#90ee90' // Light green
      case 'processing':
        return '#1e90ff' // Blue
      case 'error':
        return '#ff0000' // Red
      case 'terminated':
        return '#808080' // Gray
      default:
        return '#cccccc' // Light gray
    }
  }

  const getStatusTooltip = (terminalId: string): string => {
    const info = processInfo[terminalId]
    if (!info) return 'Terminal not initialized'
    
    const status = info.status
    if (typeof status === 'string') {
      switch (status) {
        case 'Starting':
          return 'Terminal is starting up...'
        case 'Running':
          return 'Terminal is running and ready'
        case 'Idle':
          return 'Terminal is idle and waiting for input'
        case 'Processing':
          return 'Terminal is processing commands'
        case 'Terminated':
          return 'Terminal process has terminated'
        default:
          return 'Terminal status unknown'
      }
    } else if (typeof status === 'object' && 'Error' in status) {
      return `Terminal error: ${status.Error}`
    }
    
    return 'Terminal status unknown'
  }

  if (terminals.length === 0) {
    return (
      <div className="tab-manager">
        <div className="tab-loading">
          <span>Loading terminals...</span>
        </div>
      </div>
    )
  }

  return (
    <div className="tab-manager">
      <div className="tab-list" role="tablist">
        {terminals.map((terminal, index) => {
          const isActive = terminal.id === activeTerminalId
          const status = getStatusIndicator(terminal.id)
          const statusColor = getStatusColor(status)
          const tooltip = getStatusTooltip(terminal.id)
          
          return (
            <button
              key={terminal.id}
              className={`tab ${isActive ? 'active' : ''} status-${status}`}
              role="tab"
              aria-selected={isActive}
              aria-controls={`terminal-panel-${terminal.id}`}
              tabIndex={isActive ? 0 : -1}
              title={`${tooltip} (⌘${index + 1})`}
              onClick={() => handleTabClick(terminal.id)}
            >
              <span className="tab-name">{terminal.name}</span>
              <span 
                className="tab-status-indicator"
                style={{ backgroundColor: statusColor }}
                aria-label={tooltip}
              />
              <span className="tab-shortcut">⌘{index + 1}</span>
            </button>
          )
        })}
      </div>
      
      <div className="tab-controls">
        <div className="tab-info">
          {activeTerminalId && (
            <span className="active-terminal-info">
              {terminals.find(t => t.id === activeTerminalId)?.name || 'Unknown Terminal'}
            </span>
          )}
        </div>
        
        <div className="tab-shortcuts-help" title="Keyboard shortcuts">
          <span>⌘1-5: Switch tabs | ⌘Tab: Next tab | ⌘⇧Tab: Previous tab</span>
        </div>
      </div>
    </div>
  )
}