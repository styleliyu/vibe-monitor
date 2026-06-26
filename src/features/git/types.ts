export type GitFileStatus = {
  path: string;
  indexStatus: string;
  worktreeStatus: string;
};

export type GitStatus = {
  available: boolean;
  branch?: string;
  ahead: number;
  behind: number;
  files: GitFileStatus[];
  error?: string;
};

export type GitDiff = {
  path?: string;
  text: string;
};
