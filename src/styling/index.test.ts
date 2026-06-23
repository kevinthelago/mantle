import { describe, it, expect, vi, beforeEach } from "vitest";

const mockLoadStyling = vi.hoisted(() => vi.fn());
const mockInjectCss = vi.hoisted(() => vi.fn());

vi.mock("./bridge", () => ({ loadStyling: mockLoadStyling }));
vi.mock("./injector", () => ({
  injectCss: mockInjectCss,
  removeManagedStyle: vi.fn(),
}));

import { loadAndInject } from "./index";

describe("loadAndInject", () => {
  beforeEach(() => {
    mockLoadStyling.mockReset();
    mockInjectCss.mockReset();
  });

  it("calls loadStyling and injectCss with the compiled CSS", async () => {
    mockLoadStyling.mockResolvedValue({ css: "body { color: red; }", from_cache: false });

    const css = await loadAndInject("sass", "1.69.5", "$color: red;");

    expect(mockLoadStyling).toHaveBeenCalledWith("sass", "1.69.5", "$color: red;");
    expect(mockInjectCss).toHaveBeenCalledWith("body { color: red; }");
    expect(css).toBe("body { color: red; }");
  });

  it("injects and returns last-good CSS when from_cache is true", async () => {
    mockLoadStyling.mockResolvedValue({ css: "cached {}", from_cache: true });

    const css = await loadAndInject("sass", "1.69.5", "");

    expect(mockInjectCss).toHaveBeenCalledWith("cached {}");
    expect(css).toBe("cached {}");
  });

  it("propagates errors from loadStyling without calling injectCss", async () => {
    mockLoadStyling.mockRejectedValue(new Error("network error"));

    await expect(loadAndInject("sass", "1.69.5", "")).rejects.toThrow("network error");
    expect(mockInjectCss).not.toHaveBeenCalled();
  });
});
