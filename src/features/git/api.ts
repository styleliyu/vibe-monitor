import { callTauri } from "@/shared/api/tauri";
import type { GitDiff, GitStatus } from "./types";

export function getGitStatus(workspaceId: string) {
  return callTauri<GitStatus>("git_status", { workspaceId });
}

export function getGitDiff(workspaceId: string, path?: string) {
  return callTauri<GitDiff>("git_diff", { workspaceId, path });
}

export function stageGitFile(workspaceId: string, path: string) {
  return callTauri<void>("git_stage", { workspaceId, path });
}

export function unstageGitFile(workspaceId: string, path: string) {
  return callTauri<void>("git_unstage", { workspaceId, path });
}
