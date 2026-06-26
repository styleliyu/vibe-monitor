export type Workspace = {
  id: string;
  name: string;
  path: string;
  gitRoot?: string | null;
  defaultAiEngine: "codex" | "claude" | "custom";
  createdAt: string;
  updatedAt: string;
};
