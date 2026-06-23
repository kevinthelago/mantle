import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ErrorBoundary } from "../ErrorBoundary";

function Boom(): never {
  throw new Error("intentional test error");
}

describe("ErrorBoundary", () => {
  it("renders children when there is no error", () => {
    render(
      <ErrorBoundary moduleName="clock">
        <span>OK</span>
      </ErrorBoundary>,
    );
    expect(screen.getByText("OK")).toBeTruthy();
  });

  it("catches a thrown error and shows module name", () => {
    // Suppress expected console.error output from React's error boundary.
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    render(
      <ErrorBoundary moduleName="my-module">
        <Boom />
      </ErrorBoundary>,
    );
    expect(screen.getByText(/my-module/)).toBeTruthy();
    spy.mockRestore();
  });

  it("shows the error message in the title attribute", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    render(
      <ErrorBoundary moduleName="test">
        <Boom />
      </ErrorBoundary>,
    );
    const el = screen.getByTitle("intentional test error");
    expect(el).toBeTruthy();
    spy.mockRestore();
  });

  it("logs the error to console.error", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    render(
      <ErrorBoundary moduleName="crash-module">
        <Boom />
      </ErrorBoundary>,
    );
    const calls = spy.mock.calls.flat().join(" ");
    expect(calls).toMatch(/crash-module/);
    spy.mockRestore();
  });
});
