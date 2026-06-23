import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: vi.fn().mockReturnValue({ label: "bar-DP-1" }),
}));
// Render regions as simple divs to avoid Suspense complexity.
vi.mock("../BarRegion", () => ({
  default: ({ position }: { position: string }) => (
    <div data-testid={`region-${position}`} />
  ),
}));

import { invoke } from "@tauri-apps/api/core";
import BarShell from "../BarShell";
import type { OutputConfig } from "../../../types/config";

const mockConfig: OutputConfig = {
  name: "DP-1",
  height: 36,
  layer: "top",
  exclusive: true,
  opacity: 0.95,
  keyboard: "none",
  left: { modules: ["workspaces"] },
  center: { modules: ["clock"] },
  right: { modules: ["power-menu"] },
  theme: {
    background: "#1e1e2e",
    foreground: "#cdd6f4",
    border_radius: 8,
    font: "sans-serif",
    font_size: 13,
    module_background: "transparent",
    module_padding: "0 8px",
  },
};

describe("BarShell", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders all three regions after config is fetched", async () => {
    vi.mocked(invoke).mockResolvedValue(mockConfig);
    render(<BarShell />);
    await waitFor(() => screen.getByTestId("region-left"));
    expect(screen.getByTestId("region-left")).toBeTruthy();
    expect(screen.getByTestId("region-center")).toBeTruthy();
    expect(screen.getByTestId("region-right")).toBeTruthy();
  });

  it("calls get_output_config with the output name derived from window label", async () => {
    vi.mocked(invoke).mockResolvedValue(mockConfig);
    render(<BarShell />);
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("get_output_config", {
        output: "DP-1",
      }),
    );
  });

  it("shows an error message when invoke rejects", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("backend unavailable"));
    render(<BarShell />);
    await waitFor(() => screen.getByText(/backend unavailable/));
    expect(screen.getByText(/backend unavailable/)).toBeTruthy();
  });

  it("renders nothing while config is loading", () => {
    // invoke returns a never-resolving promise.
    vi.mocked(invoke).mockReturnValue(new Promise(() => {}));
    const { container } = render(<BarShell />);
    // Only the bar-shell div may be absent; no regions should be visible.
    expect(screen.queryByTestId("region-left")).toBeNull();
    expect(container.textContent).toBe("");
  });
});
