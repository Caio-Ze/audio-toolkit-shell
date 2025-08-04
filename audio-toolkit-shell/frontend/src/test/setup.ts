import '@testing-library/jest-dom'
import { vi, beforeEach } from 'vitest'

// Mock Tauri API
const mockInvoke = vi.fn()
const mockListen = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: mockListen,
}))

// Make mocks available globally for tests
declare global {
  var mockInvoke: ReturnType<typeof vi.fn>
  var mockListen: ReturnType<typeof vi.fn>
}

globalThis.mockInvoke = mockInvoke
globalThis.mockListen = mockListen

// Reset mocks before each test
beforeEach(() => {
  mockInvoke.mockReset()
  mockListen.mockReset()
})