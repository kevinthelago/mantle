import { describe, it, expect, beforeEach } from "vitest";
import { injectCss, removeManagedStyle } from "./injector";

const STYLE_ID = "mantle-css";

describe("injectCss", () => {
  beforeEach(() => {
    document.getElementById(STYLE_ID)?.remove();
  });

  it("creates a <style> element on first call", () => {
    injectCss("body { color: red; }");
    const el = document.getElementById(STYLE_ID) as HTMLStyleElement;
    expect(el).not.toBeNull();
    expect(el.tagName).toBe("STYLE");
    expect(el.textContent).toBe("body { color: red; }");
  });

  it("replaces content in place on subsequent calls", () => {
    injectCss("body { color: red; }");
    injectCss("body { color: blue; }");
    const all = document.querySelectorAll(`#${STYLE_ID}`);
    expect(all).toHaveLength(1);
    expect((all[0] as HTMLStyleElement).textContent).toBe("body { color: blue; }");
  });

  it("appends the element to <head>", () => {
    injectCss(".foo { display: none; }");
    const el = document.head.querySelector(`#${STYLE_ID}`);
    expect(el).not.toBeNull();
  });
});

describe("removeManagedStyle", () => {
  it("removes the element when it exists", () => {
    injectCss("body {}");
    removeManagedStyle();
    expect(document.getElementById(STYLE_ID)).toBeNull();
  });

  it("is a no-op when no element exists", () => {
    expect(() => removeManagedStyle()).not.toThrow();
  });
});
