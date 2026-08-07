import { invoke } from '@tauri-apps/api/core';

/** Calls a Tauri command. The single crossing point between the UI and the Rust core. */
export function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return args === undefined ? invoke<T>(command) : invoke<T>(command, args);
}
