import { callTauri } from "@/shared/api/tauri";
import type { TerminalSession } from "./types";

export function openTerminal(workspaceId: string) {
  return callTauri<TerminalSession>("terminal_open", { workspaceId });
}

export function writeTerminal(sessionId: string, data: string) {
  return callTauri<void>("terminal_write", { sessionId, data });
}

export function resizeTerminal(sessionId: string, cols: number, rows: number) {
  return callTauri<void>("terminal_resize", { sessionId, cols, rows });
}

export function closeTerminal(sessionId: string) {
  return callTauri<void>("terminal_close", { sessionId });
}
