// @vitest-environment jsdom

import "@testing-library/jest-dom/vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { CodexPanel } from "./CodexPanel";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => vi.fn()),
}));

const invokeMock = vi.mocked(invoke);

function renderPanel(workspaceId?: string | null) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <CodexPanel workspaceId={workspaceId} />
    </QueryClientProvider>,
  );
}

describe("CodexPanel", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  afterEach(() => {
    cleanup();
  });

  it("asks for a workspace before enabling Codex controls", () => {
    renderPanel(null);

    expect(screen.getByText("Select a workspace to start a Codex session.")).toBeInTheDocument();
    expect(screen.getByRole("textbox")).toBeDisabled();
  });

  it("shows an unavailable state when Codex CLI detection fails", async () => {
    invokeMock.mockResolvedValueOnce({
      available: false,
      error: "Access is denied",
    });

    renderPanel("workspace-1");

    expect(await screen.findByText("Codex CLI unavailable")).toBeInTheDocument();
    expect(screen.getByText("Access is denied")).toBeInTheDocument();
  });

  it("starts a thread from the composer when no thread exists", async () => {
    invokeMock
      .mockResolvedValueOnce({
        available: true,
        version: "codex 1.2.3",
      })
      .mockResolvedValueOnce({
        id: "thread-1",
        workspaceId: "workspace-1",
        title: "Summarize this repo",
        status: "running",
        preview: "Summarize this repo",
      });

    renderPanel("workspace-1");

    expect(await screen.findByText("codex 1.2.3")).toBeInTheDocument();

    fireEvent.change(screen.getByRole("textbox"), {
      target: { value: "Summarize this repo" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Send to Codex" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("codex_thread_start", {
        workspaceId: "workspace-1",
        prompt: "Summarize this repo",
      });
    });
    expect(await screen.findAllByText("Summarize this repo")).toHaveLength(2);
  });
});
