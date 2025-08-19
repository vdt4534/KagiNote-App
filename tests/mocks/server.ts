import { setupServer } from 'msw/node';
import { tauriHandlers } from './handlers/tauri';

export const server = setupServer(...tauriHandlers);