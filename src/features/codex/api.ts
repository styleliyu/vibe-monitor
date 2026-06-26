import { callTauri } from "@/shared/api/tauri";
import type { CodexAvailability, CodexThreadSummary } from "./types";

export function detectCodex() {
  return callTauri<CodexAvailability>("codex_detect");
}

export function listCodexThreads(workspaceId: string) {
  return callTauri<CodexThreadSummary[]>("codex_thread_list", { workspaceId });
}

export function startCodexThread(workspaceId: string, prompt: string) {
  return callTauri<CodexThreadSummary>("codex_thread_start", { workspaceId, prompt });
}

export function sendCodexTurn(threadId: string, prompt: string) {
  return callTauri<void>("codex_turn_send", { threadId, prompt });
}

export function interruptCodexTurn(threadId: string) {
  return callTauri<void>("codex_turn_interrupt", { threadId });
}
