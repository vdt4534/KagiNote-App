import { http, HttpResponse } from 'msw';
import { vi } from 'vitest';

// Mock Tauri command responses
export const tauriMocks = {
  startTranscription: vi.fn(),
  stopTranscription: vi.fn(),
  getRealTimeResults: vi.fn(),
  exportTranscription: vi.fn(),
  getSystemInfo: vi.fn(),
};

export const tauriHandlers = [
  // These handlers can be extended as needed for testing
  // Currently empty as Tauri commands are mocked directly in tests
];