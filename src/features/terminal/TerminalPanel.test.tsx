// @vitest-environment jsdom

import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { TerminalPanel } from "./TerminalPanel";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const eventListeners = new Map<string, (event: { payload: unknown }) => void>();

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (eventName: string, handler: (event: { payload: unknown }) => void) => {
    eventListeners.set(eventName, handler);
    return vi.fn();
  }),
}));

type DataHandler = (data: string) => void;

class MockXterm {
  dataHandler?: DataHandler;
  dispose = vi.fn();
  loadAddon = vi.fn();
  onData = vi.fn((handler: DataHandler) => {
    this.dataHandler = handler;
    return { dispose: vi.fn() };
  });
  open = vi.fn();
  write = vi.fn();

  sendData(data: string) {
    this.dataHandler?.(data);
  }
}

const terminalInstances: MockXterm[] = [];

vi.mock("@xterm/xterm", () => ({
  Terminal: vi.fn(function Terminal() {
    const terminal = new MockXterm();
    terminalInstances.push(terminal);
    return terminal;
  }),
}));

vi.mock("@xterm/addon-fit", () => ({
  FitAddon: vi.fn(function FitAddon() {
    return {
    fit: vi.fn(),
    };
  }),
}));

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);

describe("TerminalPanel", () => {
  beforeEach(() => {
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
    invokeMock.mockReset();
    listenMock.mockClear();
    eventListeners.clear();
    terminalInstances.length = 0;
  });

  afterEach(() => {
    cleanup();
    Reflect.deleteProperty(window, "__TAURI_INTERNALS__");
  });

  it("requires a workspace before opening a terminal", () => {
    render(<TerminalPanel workspaceId={null} />);

    expect(screen.getByText("Select a workspace to open a terminal.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Open terminal" })).toBeDisabled();
  });

  it("opens a terminal session for the selected workspace", async () => {
    invokeMock.mockResolvedValueOnce({
      id: "terminal-1",
      workspaceId: "workspace-1",
      cwd: "D:\\AIdeas\\CodingMonitor",
      shell: "powershell.exe",
      cols: 80,
      rows: 24,
      createdAt: "2026-06-26T00:00:00Z",
    });

    render(<TerminalPanel workspaceId="workspace-1" />);

    fireEvent.click(screen.getByRole("button", { name: "Open terminal" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("terminal_open", {
        workspaceId: "workspace-1",
      });
    });
    expect(await screen.findByText("powershell.exe")).toBeInTheDocument();
    expect(terminalInstances).toHaveLength(1);
  });

  it("sends terminal keyboard input to the active backend session", async () => {
    invokeMock
      .mockResolvedValueOnce({
        id: "terminal-1",
        workspaceId: "workspace-1",
        cwd: "D:\\AIdeas\\CodingMonitor",
        shell: "powershell.exe",
        cols: 80,
        rows: 24,
        createdAt: "2026-06-26T00:00:00Z",
      })
      .mockResolvedValueOnce(undefined);

    render(<TerminalPanel workspaceId="workspace-1" />);
    fireEvent.click(screen.getByRole("button", { name: "Open terminal" }));
    await screen.findByText("powershell.exe");

    terminalInstances[0].sendData("git status\r");

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("terminal_write", {
        sessionId: "terminal-1",
        data: "git status\r",
      });
    });
  });

  it("writes matching backend output events into the terminal", async () => {
    invokeMock.mockResolvedValueOnce({
      id: "terminal-1",
      workspaceId: "workspace-1",
      cwd: "D:\\AIdeas\\CodingMonitor",
      shell: "powershell.exe",
      cols: 80,
      rows: 24,
      createdAt: "2026-06-26T00:00:00Z",
    });

    render(<TerminalPanel workspaceId="workspace-1" />);
    fireEvent.click(screen.getByRole("button", { name: "Open terminal" }));
    await screen.findByText("powershell.exe");

    eventListeners.get("terminal://output")?.({
      payload: {
        sessionId: "terminal-1",
        data: "On branch feature/terminal-mvp\r\n",
      },
    });

    expect(terminalInstances[0].write).toHaveBeenCalledWith(
      "On branch feature/terminal-mvp\r\n",
    );
  });
});
