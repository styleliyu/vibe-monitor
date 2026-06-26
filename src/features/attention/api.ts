import { callTauri } from "@/shared/api/tauri";
import type { AttentionItem, CreateAttentionItem } from "./types";

export function listAttentionItems(workspaceId?: string | null) {
  return callTauri<AttentionItem[]>("attention_list", { workspaceId });
}

export function createAttentionItem(input: CreateAttentionItem) {
  return callTauri<AttentionItem>("attention_create", { input });
}

export function resolveAttentionItem(id: string) {
  return callTauri<void>("attention_resolve", { id });
}
