export type TerminalSession = {
  id: string;
  workspaceId: string;
  cwd: string;
  shell: string;
  cols: number;
  rows: number;
  createdAt: string;
};

export type TerminalOutputEvent = {
  sessionId: string;
  data: string;
};
