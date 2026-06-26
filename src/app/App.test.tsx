// @vitest-environment jsdom

import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach } from "vitest";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App";

vi.mock("@xterm/xterm", () => ({
  Terminal: vi.fn(function Terminal() {
    return {
      dispose: vi.fn(),
      loadAddon: vi.fn(),
      onData: vi.fn(),
      open: vi.fn(),
      write: vi.fn(),
    };
  }),
}));

vi.mock("@xterm/addon-fit", () => ({
  FitAddon: vi.fn(function FitAddon() {
    return {
      fit: vi.fn(),
    };
  }),
}));

describe("App", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.className = "";
    document.documentElement.removeAttribute("lang");
  });

  afterEach(() => {
    cleanup();
  });

  it("renders simplified Chinese by default with dark mode enabled", () => {
    render(<App />);

    expect(screen.getByText("vibe-monitor")).toBeInTheDocument();
    expect(screen.getByText("工作区")).toBeInTheDocument();
    expect(screen.getByText("Codex 会话")).toBeInTheDocument();
    expect(screen.getByText("注意力队列")).toBeInTheDocument();
    expect(screen.getByText("终端")).toBeInTheDocument();
    expect(screen.getByText("Git")).toBeInTheDocument();
    expect(document.documentElement).toHaveClass("dark");
    expect(document.documentElement).toHaveAttribute("lang", "zh-CN");
  });

  it("switches the visible app language to English from settings", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "设置" }));
    fireEvent.click(screen.getByRole("button", { name: "English" }));

    expect(screen.getByText("Workspaces")).toBeInTheDocument();
    expect(screen.getByText("Codex Session")).toBeInTheDocument();
    expect(screen.getByText("Attention Queue")).toBeInTheDocument();
    expect(screen.getByText("Terminal")).toBeInTheDocument();
    expect(document.documentElement).toHaveAttribute("lang", "en");
  });
});
