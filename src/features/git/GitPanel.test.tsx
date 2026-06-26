// @vitest-environment jsdom

import "@testing-library/jest-dom/vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { GitPanel } from "./GitPanel";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

function renderGitPanel(workspaceId: string | null) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <GitPanel workspaceId={workspaceId} />
    </QueryClientProvider>,
  );
}

describe("GitPanel", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  afterEach(() => {
    cleanup();
  });

  it("requires a workspace before showing Git status", () => {
    renderGitPanel(null);

    expect(screen.getByText("Select a workspace to inspect Git changes.")).toBeInTheDocument();
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("shows unavailable state for non-Git workspaces", async () => {
    invokeMock.mockResolvedValueOnce({
      available: false,
      ahead: 0,
      behind: 0,
      files: [],
      error: "workspace is not a Git repository",
    });

    renderGitPanel("workspace-1");

    expect(await screen.findByText("Git unavailable")).toBeInTheDocument();
    expect(screen.getByText("workspace is not a Git repository")).toBeInTheDocument();
  });

  it("loads a selected file diff", async () => {
    invokeMock
      .mockResolvedValueOnce({
        available: true,
        branch: "feature/git-status-diff",
        ahead: 1,
        behind: 0,
        files: [{ path: "src/app/App.tsx", indexStatus: " ", worktreeStatus: "M" }],
      })
      .mockResolvedValueOnce({
        path: "src/app/App.tsx",
        text: "diff --git a/src/app/App.tsx b/src/app/App.tsx",
      });

    renderGitPanel("workspace-1");

    fireEvent.click(await screen.findByRole("button", { name: "src/app/App.tsx" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("git_diff", {
        workspaceId: "workspace-1",
        path: "src/app/App.tsx",
      });
    });
    expect(await screen.findByText(/diff --git/)).toBeInTheDocument();
  });

  it("stages and unstages the selected file then refreshes status", async () => {
    const status = {
      available: true,
      branch: "main",
      ahead: 0,
      behind: 0,
      files: [{ path: "src/app/App.tsx", indexStatus: " ", worktreeStatus: "M" }],
    };
    invokeMock
      .mockResolvedValueOnce(status)
      .mockResolvedValueOnce({ path: "src/app/App.tsx", text: "diff text" })
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(status)
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(status);

    renderGitPanel("workspace-1");
    fireEvent.click(await screen.findByRole("button", { name: "src/app/App.tsx" }));

    fireEvent.click(await screen.findByRole("button", { name: "Stage" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("git_stage", {
        workspaceId: "workspace-1",
        path: "src/app/App.tsx",
      });
    });

    fireEvent.click(await screen.findByRole("button", { name: "Unstage" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("git_unstage", {
        workspaceId: "workspace-1",
        path: "src/app/App.tsx",
      });
    });
  });
});
