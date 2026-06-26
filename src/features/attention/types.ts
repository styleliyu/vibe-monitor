export type AttentionKind =
  | "approval"
  | "blocked"
  | "failed"
  | "done"
  | "unread"
  | "info";

export type AttentionPriority = 0 | 1 | 2 | 3;

export type AttentionItem = {
  id: string;
  workspaceId: string;
  sessionId?: string | null;
  kind: AttentionKind;
  priority: AttentionPriority;
  title: string;
  summary: string;
  actionLabel?: string | null;
  actionRef?: string | null;
  createdAt: string;
  resolvedAt?: string | null;
};

export type CreateAttentionItem = {
  workspaceId: string;
  sessionId?: string | null;
  kind: AttentionKind;
  priority: AttentionPriority;
  title: string;
  summary: string;
  actionLabel?: string | null;
  actionRef?: string | null;
};
