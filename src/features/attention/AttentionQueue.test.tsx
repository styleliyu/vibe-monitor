// @vitest-environment jsdom

import "@testing-library/jest-dom/vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { AttentionQueue } from "./AttentionQueue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

function renderQueue(workspaceId?: string) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <AttentionQueue workspaceId={workspaceId} />
    </QueryClientProvider>,
  );
}

describe("AttentionQueue", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  afterEach(() => {
    cleanup();
  });

  it("shows an empty active queue", async () => {
    invokeMock.mockResolvedValueOnce([]);

    renderQueue("workspace-1");

    expect(await screen.findByText("No attention items.")).toBeInTheDocument();
    expect(screen.getByText("0 active")).toBeInTheDocument();
  });

  it("renders attention items with kind and priority", async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: "attention-1",
        workspaceId: "workspace-1",
        sessionId: null,
        kind: "failed",
        priority: 3,
        title: "Build failed",
        summary: "npm run build exited with code 1",
        actionLabel: "Open log",
        actionRef: "session-1",
        createdAt: "2026-06-26T00:00:00Z",
        resolvedAt: null,
      },
    ]);

    renderQueue("workspace-1");

    expect(await screen.findByText("Build failed")).toBeInTheDocument();
    expect(screen.getByText("failed")).toBeInTheDocument();
    expect(screen.getByText("P3")).toBeInTheDocument();
    expect(screen.getByText("1 active")).toBeInTheDocument();
  });

  it("resolves an attention item through the typed Tauri API", async () => {
    invokeMock
      .mockResolvedValueOnce([
        {
          id: "attention-1",
          workspaceId: "workspace-1",
          sessionId: null,
          kind: "failed",
          priority: 3,
          title: "Build failed",
          summary: "npm run build exited with code 1",
          actionLabel: null,
          actionRef: null,
          createdAt: "2026-06-26T00:00:00Z",
          resolvedAt: null,
        },
      ])
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([]);

    renderQueue("workspace-1");

    fireEvent.click(await screen.findByRole("button", { name: "Resolve Build failed" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("attention_resolve", {
        id: "attention-1",
      });
    });
    expect(await screen.findByText("No attention items.")).toBeInTheDocument();
  });
});
