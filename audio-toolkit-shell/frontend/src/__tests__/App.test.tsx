import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import App from '../App'

// Mock the store hooks
const mockInitialize = vi.fn()

vi.mock('../store/app-store', () => ({
  useAuthState: vi.fn(() => ({
    authStatus: null,
    versionStatus: null,
    serverMessage: null,
    error: null,
  })),
  useUIState: vi.fn(() => ({
    isLoading: true,
    isInitialized: false,
  })),
  useAppActions: vi.fn(() => ({
    initialize: mockInitialize,
  })),
}))

describe('App', () => {
  it('should render without crashing', () => {
    render(<App />)
    expect(screen.getByText('Audio Toolkit Shell')).toBeInTheDocument()
  })

  it('should call initialize on mount', () => {
    render(<App />)
    expect(mockInitialize).toHaveBeenCalled()
  })
})