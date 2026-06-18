import { invoke } from '@tauri-apps/api/core';

let _platform = 'unknown';

export async function initPlatform(): Promise<void> {
  _platform = await invoke<string>('get_platform');
}

export function isIOS(): boolean {
  return _platform === 'ios';
}
