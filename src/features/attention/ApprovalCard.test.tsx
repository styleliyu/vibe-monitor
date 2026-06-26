// @vitest-environment jsdom

import "@testing-library/jest-dom/vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ApprovalCard } from "./ApprovalCard";

describe("ApprovalCard", () => {
  it("renders approval context and action buttons", () => {
    const approve = vi.fn();
    const reject = vi.fn();

    render(
      <ApprovalCard
        approval={{
          command: "npm test",
          requestedScope: "workspace-write",
          workspacePath: "D:\\AIdeas\\CodingMonitor",
        }}
        onApprove={approve}
        onReject={reject}
      />,
    );

    expect(screen.getByText("npm test")).toBeInTheDocument();
    expect(screen.getByText("workspace-write")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Approve Codex request" }));
    fireEvent.click(screen.getByRole("button", { name: "Reject Codex request" }));

    expect(approve).toHaveBeenCalledTimes(1);
    expect(reject).toHaveBeenCalledTimes(1);
  });
});
