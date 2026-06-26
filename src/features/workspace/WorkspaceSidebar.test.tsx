// @vitest-environment jsdom

import "@testing-library/jest-dom/vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { WorkspaceSidebar } from "./WorkspaceSidebar";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

function renderSidebar() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <WorkspaceSidebar />
    </QueryClientProvider>,
  );
}

describe("WorkspaceSidebar", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  afterEach(() => {
    cleanup();
  });

  it("shows an empty state when no workspaces exist", async () => {
    invokeMock.mockResolvedValueOnce([]);

    renderSidebar();

    expect(
      await screen.findByText("Add a workspace to start monitoring AI development work."),
    ).toBeInTheDocument();
  });

  it("adds a workspace through the typed Tauri API", async () => {
    invokeMock
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce({
        id: "workspace-1",
        name: "CodingMonitor",
        path: "D:\\AIdeas\\CodingMonitor",
        gitRoot: "D:\\AIdeas\\CodingMonitor",
        defaultAiEngine: "codex",
        createdAt: "2026-06-26T00:00:00Z",
        updatedAt: "2026-06-26T00:00:00Z",
      })
      .mockResolvedValueOnce([
        {
          id: "workspace-1",
          name: "CodingMonitor",
          path: "D:\\AIdeas\\CodingMonitor",
          gitRoot: "D:\\AIdeas\\CodingMonitor",
          defaultAiEngine: "codex",
          createdAt: "2026-06-26T00:00:00Z",
          updatedAt: "2026-06-26T00:00:00Z",
        },
      ]);

    renderSidebar();

    fireEvent.change(await screen.findByLabelText("Workspace path"), {
      target: { value: "D:\\AIdeas\\CodingMonitor" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add workspace" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("workspace_add", {
        path: "D:\\AIdeas\\CodingMonitor",
        name: undefined,
      });
    });
    expect(await screen.findByText("CodingMonitor")).toBeInTheDocument();
  });
});
