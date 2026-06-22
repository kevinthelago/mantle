import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useLauncher } from "../hooks/useLauncher";

// Mock the Tauri API bridge.
vi.mock("../api", () => ({
  api: {
    search: vi.fn().mockResolvedValue([
      {
        entry: {
          id: "firefox",
          name: "Firefox",
          description: "Web browser",
          icon: undefined,
          exec: "firefox %u",
          terminal: false,
          categories: ["Network"],
          keywords: ["browser"],
          desktopFile: "/usr/share/applications/firefox.desktop",
        },
        score: 0.9,
        launchCount: 5,
      },
    ]),
    launch: vi.fn().mockResolvedValue(undefined),
    runCommand: vi.fn().mockResolvedValue(undefined),
    close: vi.fn().mockResolvedValue(undefined),
  },
}));

describe("useLauncher", () => {
  it("seeds results on mount", async () => {
    const { result } = renderHook(() => useLauncher());
    await waitFor(() => expect(result.current.results).toHaveLength(1));
    expect(result.current.results[0].entry.name).toBe("Firefox");
  });

  it("switches to run mode on > prefix", async () => {
    const { result } = renderHook(() => useLauncher());
    act(() => result.current.setQuery(">ls -la"));
    expect(result.current.mode).toBe("run");
    expect(result.current.results).toHaveLength(0);
  });

  it("selectNext/selectPrev clamps to bounds", async () => {
    const { result } = renderHook(() => useLauncher());
    await waitFor(() => expect(result.current.results).toHaveLength(1));
    act(() => result.current.selectNext());
    expect(result.current.selectedIndex).toBe(0); // clamped at max
    act(() => result.current.selectPrev());
    expect(result.current.selectedIndex).toBe(0); // clamped at 0
  });

  it("confirm in run mode calls runCommand then close", async () => {
    const { api } = await import("../api");
    const { result } = renderHook(() => useLauncher());
    act(() => result.current.setQuery(">echo hello"));
    await act(async () => result.current.confirm());
    expect(api.runCommand).toHaveBeenCalledWith("echo hello");
    expect(api.close).toHaveBeenCalled();
  });

  it("confirm in apps mode launches selected entry", async () => {
    const { api } = await import("../api");
    const { result } = renderHook(() => useLauncher());
    await waitFor(() => expect(result.current.results).toHaveLength(1));
    await act(async () => result.current.confirm());
    expect(api.launch).toHaveBeenCalledWith("firefox");
    expect(api.close).toHaveBeenCalled();
  });

  it("reset clears state", async () => {
    const { result } = renderHook(() => useLauncher());
    act(() => result.current.setQuery("fire"));
    act(() => result.current.reset());
    expect(result.current.query).toBe("");
    expect(result.current.mode).toBe("apps");
  });
});
