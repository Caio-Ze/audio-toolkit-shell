// STORE-AWARE CONTAINER COMPONENT
// This component uses store hooks and passes data to IsolatedTerminalPane as props
// Separates store integration from terminal rendering to prevent black screen

import { useTerminalState, useAppActions } from '../store/app-store'
import IsolatedTerminalPane from './IsolatedTerminalPane'

interface TerminalContainerProps {
  terminalId: string
  isActive: boolean
  onResize?: (cols: number, rows: number) => void
}

export default function TerminalContainer({ terminalId, isActive, onResize }: TerminalContainerProps) {
  // Store hooks are isolated to this container component
  const { terminals, processInfo } = useTerminalState()
  const { updateProcessStatus } = useAppActions()

  // Find terminal and process info
  const terminal = terminals.find(t => t.id === terminalId)
  const terminalProcessInfo = processInfo[terminalId]

  // Handle process status updates
  const handleProcessStatusUpdate = (terminalId: string, status: any) => {
    updateProcessStatus(terminalId, status)
  }

  // Pass all data as props to isolated terminal
  return (
    <IsolatedTerminalPane
      terminalId={terminalId}
      isActive={isActive}
      onResize={onResize}
      terminal={terminal}
      processInfo={terminalProcessInfo}
      onProcessStatusUpdate={handleProcessStatusUpdate}
    />
  )
}