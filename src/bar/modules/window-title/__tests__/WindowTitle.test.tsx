import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

import { invoke } from "@tauri-apps/api/core";
import WindowTitle from "../WindowTitle";
import type { FocusedWindow } from "../../../../types/config";

describe("WindowTitle", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the focused window title", async () => {
    const focused: FocusedWindow = { title: "Firefox", app_id: "firefox" };
    vi.mocked(invoke).mockResolvedValue(focused);
    render(<WindowTitle />);
    await waitFor(() => screen.getByText("Firefox"));
    expect(screen.getByText("Firefox")).toBeTruthy();
  });

  it("renders nothing when title is null", async () => {
    const focused: FocusedWindow = { title: null, app_id: null };
    vi.mocked(invoke).mockResolvedValue(focused);
    const { container } = render(<WindowTitle />);
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("get_focused_window"),
    );
    // Component returns null — no child element should appear in the container.
    expect(container.firstElementChild).toBeNull();
  });

  it("renders nothing when title is an empty string", async () => {
    const focused: FocusedWindow = { title: "", app_id: "empty-app" };
    vi.mocked(invoke).mockResolvedValue(focused);
    const { container } = render(<WindowTitle />);
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("get_focused_window"),
    );
    expect(container.firstElementChild).toBeNull();
  });

  it("sets the title attribute for tooltip on overflow", async () => {
    const longTitle = "A very long window title that may overflow";
    const focused: FocusedWindow = { title: longTitle, app_id: "some-app" };
    vi.mocked(invoke).mockResolvedValue(focused);
    render(<WindowTitle />);
    await waitFor(() => screen.getByTitle(longTitle));
    expect(screen.getByTitle(longTitle)).toBeTruthy();
  });

  it("calls get_focused_window on mount", async () => {
    vi.mocked(invoke).mockResolvedValue({ title: null, app_id: null });
    render(<WindowTitle />);
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("get_focused_window"),
    );
  });
});
