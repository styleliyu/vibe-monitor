// @vitest-environment jsdom

import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

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
