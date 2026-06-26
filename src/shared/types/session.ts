export type SessionEngine = "codex" | "terminal" | "browser" | "external";

export type SessionStatus =
  | "idle"
  | "running"
  | "blocked"
  | "failed"
  | "review"
  | "done";

export type SessionSummary = {
  id: string;
  workspaceId: string;
  engine: SessionEngine;
  title: string;
  status: SessionStatus;
  sourceId?: string;
  createdAt: string;
  updatedAt: string;
};
