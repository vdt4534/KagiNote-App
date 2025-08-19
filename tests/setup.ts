import '@testing-library/jest-dom';
import { vi } from 'vitest';
import { server } from './mocks/server';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
  emit: vi.fn()
}));

// Setup MSW server for API mocking
beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

// Global test utilities
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn()
}));

global.AudioContext = vi.fn().mockImplementation(() => ({
  createAnalyser: vi.fn(),
  createGain: vi.fn(),
  createMediaStreamSource: vi.fn(),
  close: vi.fn(),
  state: 'running'
}));

global.navigator = {
  ...global.navigator,
  mediaDevices: {
    getUserMedia: vi.fn().mockResolvedValue({
      getTracks: vi.fn().mockReturnValue([]),
      getAudioTracks: vi.fn().mockReturnValue([])
    })
  }
};

// Test utilities
export const waitFor = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

export const flushPromises = () => new Promise(setImmediate);