import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

import { invoke } from "@tauri-apps/api/core";
import Clock from "../Clock";
import type { ClockConfig } from "../../../../types/config";

const mockConfig: ClockConfig = {
  format: "%H:%M",
  interval: 60000,
  timezone: null,
};

describe("Clock", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_clock_config") return Promise.resolve(mockConfig);
      if (cmd === "clock_tick") return Promise.resolve("14:30");
      return Promise.resolve(null);
    });
  });

  it("renders the time string returned by clock_tick", async () => {
    render(<Clock />);
    await waitFor(() => screen.getByText("14:30"));
    expect(screen.getByText("14:30")).toBeTruthy();
  });

  it("renders nothing while config is still loading", () => {
    vi.mocked(invoke).mockReturnValue(new Promise(() => {}));
    render(<Clock />);
    expect(screen.queryByRole("time")).toBeNull();
  });

  it("calls get_clock_config on mount", async () => {
    render(<Clock />);
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("get_clock_config"),
    );
  });

  it("calls clock_tick after config loads", async () => {
    render(<Clock />);
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("clock_tick"),
    );
  });

  it("renders a <time> element with the formatted time", async () => {
    render(<Clock />);
    const el = await waitFor(() => screen.getByRole("time"));
    expect(el.textContent).toBe("14:30");
  });

  it("handles clock_tick errors gracefully (stays silent)", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_clock_config") return Promise.resolve(mockConfig);
      if (cmd === "clock_tick") return Promise.reject(new Error("IPC error"));
      return Promise.resolve(null);
    });
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    render(<Clock />);
    // After the tick fails, the component should render nothing (time stays "").
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("clock_tick"));
    expect(screen.queryByRole("time")).toBeNull();
    spy.mockRestore();
  });
});
