// @vitest-environment jsdom

import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
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
  it("renders the fixed MVP cockpit regions", () => {
    render(<App />);

    expect(screen.getByText("vibe-monitor")).toBeInTheDocument();
    expect(screen.getByText("Workspaces")).toBeInTheDocument();
    expect(screen.getByText("Codex Session")).toBeInTheDocument();
    expect(screen.getByText("Attention Queue")).toBeInTheDocument();
    expect(screen.getByText("Terminal")).toBeInTheDocument();
    expect(screen.getByText("Git")).toBeInTheDocument();
  });
});
