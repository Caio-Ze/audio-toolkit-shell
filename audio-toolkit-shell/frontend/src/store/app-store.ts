import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import type { 
  TerminalConfig, 
  ProcessInfo,
  ProcessStatus 
} from '../types'
import type { AuthStatus, VersionStatus } from '../types/auth'

interface AppState {
  // Authentication state
  authStatus: AuthStatus | null
  versionStatus: VersionStatus | null
  serverMessage: string | null
  isLoading: boolean
  error: string | null

  // Terminal state
  terminals: TerminalConfig[]
  activeTerminalId: string | null
  processInfo: Record<string, ProcessInfo>

  // UI state
  isInitialized: boolean
  showStatusBar: boolean
}

interface AppActions {
  // Authentication actions
  setAuthStatus: (status: AuthStatus | null) => void
  setVersionStatus: (status: VersionStatus | null) => void
  setServerMessage: (message: string | null) => void
  setLoading: (loading: boolean) => void
  setError: (error: string | null) => void

  // Terminal actions
  setTerminals: (terminals: TerminalConfig[]) => void
  setActiveTerminal: (terminalId: string | null) => void
  updateProcessInfo: (terminalId: string, info: ProcessInfo) => void
  updateProcessStatus: (terminalId: string, status: ProcessStatus) => void

  // UI actions
  setInitialized: (initialized: boolean) => void
  setShowStatusBar: (show: boolean) => void

  // Combined actions
  reset: () => void
  initialize: () => Promise<void>
}

type AppStore = AppState & AppActions

const initialState: AppState = {
  authStatus: null,
  versionStatus: null,
  serverMessage: null,
  isLoading: true,
  error: null,
  terminals: [],
  activeTerminalId: null,
  processInfo: {},
  isInitialized: false,
  showStatusBar: true,
}

export const useAppStore = create<AppStore>()(
  devtools(
    (set, get) => ({
      ...initialState,

      // Authentication actions
      setAuthStatus: (status) => set({ authStatus: status }),
      setVersionStatus: (status) => set({ versionStatus: status }),
      setServerMessage: (message) => set({ serverMessage: message }),
      setLoading: (loading) => set({ isLoading: loading }),
      setError: (error) => set({ error }),

      // Terminal actions
      setTerminals: (terminals) => {
        set({ terminals })
        // Set first terminal as active if none is selected
        const { activeTerminalId } = get()
        if (!activeTerminalId && terminals.length > 0) {
          set({ activeTerminalId: terminals[0].id })
        }
      },
      
      setActiveTerminal: (terminalId) => set({ activeTerminalId: terminalId }),
      
      updateProcessInfo: (terminalId, info) => 
        set((state) => ({
          processInfo: {
            ...state.processInfo,
            [terminalId]: info
          }
        })),
      
      updateProcessStatus: (terminalId, status) =>
        set((state) => ({
          processInfo: {
            ...state.processInfo,
            [terminalId]: {
              ...state.processInfo[terminalId],
              status
            }
          }
        })),

      // UI actions
      setInitialized: (initialized) => set({ isInitialized: initialized }),
      setShowStatusBar: (show) => set({ showStatusBar: show }),

      // Combined actions
      reset: () => set(initialState),
      
      initialize: async () => {
        const { setLoading, setError, setInitialized } = get()
        
        try {
          setLoading(true)
          setError(null)
          
          // Import Tauri API functions
          const { validateUserAccess, getTerminalConfigs } = await import('../utils/tauri-api')
          
          // Validate authentication
          const authResponse = await validateUserAccess()
          set({
            authStatus: authResponse.auth_status,
            versionStatus: authResponse.version_status,
            serverMessage: authResponse.server_message || null
          })

          // Check for version update requirements
          if (typeof authResponse.version_status === 'object' && 'UpdateRequired' in authResponse.version_status) {
            setError(`Update required to version ${authResponse.version_status.UpdateRequired}`)
            return
          }

          // Check authentication
          if (!authResponse.auth_status.is_authenticated) {
            setError('Access denied. Please contact your administrator.')
            return
          }

          // Load terminal configurations
          const terminals = await getTerminalConfigs()
          set({ terminals })

          // Initialize mock process info for development
          const mockProcessInfo: Record<string, ProcessInfo> = {}
          terminals.forEach((terminal, index) => {
            const statuses: ProcessStatus[] = ['Running', 'Idle', 'Processing', { Error: 'Connection failed' }, 'Starting']
            mockProcessInfo[terminal.id] = {
              terminal_id: terminal.id,
              status: statuses[index % statuses.length],
              pid: 1000 + index,
              started_at: new Date().toISOString(),
              last_activity: new Date().toISOString()
            }
          })
          set({ processInfo: mockProcessInfo })

          setInitialized(true)
        } catch (error) {
          console.error('Failed to initialize app:', error)
          setError('Failed to initialize application. Please try again.')
        } finally {
          setLoading(false)
        }
      }
    }),
    {
      name: 'audio-toolkit-shell-store',
    }
  )
)

// Selectors for common state combinations
export const useAuthState = () => useAppStore((state) => ({
  authStatus: state.authStatus,
  versionStatus: state.versionStatus,
  serverMessage: state.serverMessage,
  isLoading: state.isLoading,
  error: state.error,
}))

export const useTerminalState = () => useAppStore((state) => ({
  terminals: state.terminals,
  activeTerminalId: state.activeTerminalId,
  processInfo: state.processInfo,
}))

export const useUIState = () => useAppStore((state) => ({
  isInitialized: state.isInitialized,
  showStatusBar: state.showStatusBar,
  isLoading: state.isLoading,
  error: state.error,
}))

// Action selectors
export const useAppActions = () => useAppStore((state) => ({
  setAuthStatus: state.setAuthStatus,
  setVersionStatus: state.setVersionStatus,
  setServerMessage: state.setServerMessage,
  setLoading: state.setLoading,
  setError: state.setError,
  setTerminals: state.setTerminals,
  setActiveTerminal: state.setActiveTerminal,
  updateProcessInfo: state.updateProcessInfo,
  updateProcessStatus: state.updateProcessStatus,
  setInitialized: state.setInitialized,
  setShowStatusBar: state.setShowStatusBar,
  reset: state.reset,
  initialize: state.initialize,
}))