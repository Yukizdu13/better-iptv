import { info, warn, error, debug, attachConsole } from '@tauri-apps/plugin-log';

// Attach console in development mode
// This makes Rust logs appear in the browser console
if (import.meta.env.DEV) {
  attachConsole();
}

// Export logger functions for use throughout the app
export const logger = {
  info,
  warn,
  error,
  debug,
};
