import { callTauri } from "@/shared/api/tauri";
import type { Workspace } from "./types";

export function listWorkspaces() {
  return callTauri<Workspace[]>("workspace_list");
}

export function addWorkspace(path: string, name?: string) {
  return callTauri<Workspace>("workspace_add", { path, name });
}

export function removeWorkspace(id: string) {
  return callTauri<void>("workspace_remove", { id });
}
