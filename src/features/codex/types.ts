export type CodexAvailability = {
  available: boolean;
  version?: string | null;
  error?: string | null;
};

export type CodexThreadStatus = "idle" | "running" | "blocked" | "failed" | "done";

export type CodexThreadSummary = {
  id: string;
  workspaceId: string;
  title: string;
  status: CodexThreadStatus;
  preview?: string | null;
  createdAt?: string | null;
  updatedAt?: string | null;
};

export type CodexEventKind =
  | "user"
  | "assistant"
  | "tool"
  | "command"
  | "approval"
  | "unknown";

export type CodexEvent = {
  id: string;
  kind: CodexEventKind;
  title: string;
  body?: string;
  raw?: unknown;
};

export type ApprovalContext = {
  command: string;
  workspacePath: string;
  requestedScope: string;
};
