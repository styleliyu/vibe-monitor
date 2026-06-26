import { invoke } from "@tauri-apps/api/core";

export function callTauri<T>(command: string, args?: Record<string, unknown>) {
  return invoke<T>(command, args);
}
