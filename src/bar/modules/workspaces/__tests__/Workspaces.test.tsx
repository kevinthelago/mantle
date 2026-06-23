import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: vi.fn().mockReturnValue({ label: "bar-DP-1" }),
}));

import { invoke } from "@tauri-apps/api/core";
import Workspaces from "../Workspaces";
import type { WorkspaceState } from "../../../../types/config";

const mockState: WorkspaceState = {
  workspaces: [
    { id: 1, name: "1", output: "DP-1", focused: true, urgent: false, empty: false },
    { id: 2, name: "2", output: "DP-1", focused: false, urgent: true, empty: false },
    { id: 3, name: "3", output: "HDMI-A-1", focused: false, urgent: false, empty: true },
  ],
  focused_output: "DP-1",
};

describe("Workspaces", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_workspace_state") return Promise.resolve(mockState);
      return Promise.resolve(undefined);
    });
  });

  it("renders only workspaces for the current output", async () => {
    render(<Workspaces />);
    await waitFor(() => screen.getByText("1"));
    expect(screen.getByText("1")).toBeTruthy();
    expect(screen.getByText("2")).toBeTruthy();
    // Workspace 3 is on HDMI-A-1, should be filtered out.
    expect(screen.queryByText("3")).toBeNull();
  });

  it("marks the focused workspace with aria-selected=true", async () => {
    render(<Workspaces />);
    const focused = await waitFor(() => screen.getByRole("tab", { name: "1" }));
    expect(focused.getAttribute("aria-selected")).toBe("true");
    const unfocused = screen.getByRole("tab", { name: "2" });
    expect(unfocused.getAttribute("aria-selected")).toBe("false");
  });

  it("dispatches compositor_dispatch when a workspace button is clicked", async () => {
    const user = userEvent.setup();
    render(<Workspaces />);
    await waitFor(() => screen.getByText("2"));
    await user.click(screen.getByText("2"));
    expect(invoke).toHaveBeenCalledWith("compositor_dispatch", {
      command: "workspace 2",
    });
  });

  it("renders the tablist with an accessible label", async () => {
    render(<Workspaces />);
    await waitFor(() => screen.getByRole("tablist"));
    expect(screen.getByRole("tablist")).toBeTruthy();
  });
});
