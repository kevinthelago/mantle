import { describe, it, expect, vi, beforeEach } from "vitest";

const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));

import { loadStyling } from "./bridge";

describe("loadStyling", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("invokes the load_styling command with the correct args", async () => {
    const expected = { css: "body {}", from_cache: false };
    mockInvoke.mockResolvedValue(expected);

    const result = await loadStyling("sass", "1.69.5", "$color: red;");

    expect(mockInvoke).toHaveBeenCalledOnce();
    expect(mockInvoke).toHaveBeenCalledWith("load_styling", {
      tool: "sass",
      version: "1.69.5",
      input: "$color: red;",
    });
    expect(result).toEqual(expected);
  });

  it("propagates invoke errors to the caller", async () => {
    mockInvoke.mockRejectedValue(new Error("package 'evil' is not in the allowlist"));
    await expect(loadStyling("evil", "1.0.0", "")).rejects.toThrow("allowlist");
  });

  it("reflects from_cache=true when the backend serves last-good CSS", async () => {
    mockInvoke.mockResolvedValue({ css: "body { color: red; }", from_cache: true });
    const result = await loadStyling("sass", "1.0.0", "");
    expect(result.from_cache).toBe(true);
  });
});
