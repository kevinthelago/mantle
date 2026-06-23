import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

import { invoke } from "@tauri-apps/api/core";
import PowerMenu from "../PowerMenu";
import type { Config } from "../../../../types/config";

const mockConfig = {
  power_menu: {
    actions: ["lock", "suspend", "reboot", "poweroff"],
  },
} as unknown as Config;

describe("PowerMenu", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_config") return Promise.resolve(mockConfig);
      if (cmd === "power_action") return Promise.resolve(undefined);
      return Promise.resolve(undefined);
    });
  });

  it("renders the trigger button", async () => {
    render(<PowerMenu />);
    await waitFor(() => screen.getByRole("button", { name: "Power menu" }));
    expect(screen.getByRole("button", { name: "Power menu" })).toBeTruthy();
  });

  it("dropdown is closed by default", async () => {
    render(<PowerMenu />);
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("get_config"),
    );
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("opens the dropdown when trigger is clicked", async () => {
    const user = userEvent.setup();
    render(<PowerMenu />);
    await waitFor(() => screen.getByRole("button", { name: "Power menu" }));
    await user.click(screen.getByRole("button", { name: "Power menu" }));
    expect(screen.getByRole("menu")).toBeTruthy();
  });

  it("shows only the configured actions", async () => {
    const user = userEvent.setup();
    render(<PowerMenu />);
    await waitFor(() => screen.getByRole("button", { name: "Power menu" }));
    await user.click(screen.getByRole("button", { name: "Power menu" }));
    expect(screen.getByRole("menuitem", { name: /Lock/ })).toBeTruthy();
    expect(screen.getByRole("menuitem", { name: /Reboot/ })).toBeTruthy();
    // Hibernate is not in the mock config
    expect(screen.queryByRole("menuitem", { name: /Hibernate/ })).toBeNull();
  });

  it("closes the dropdown when Escape is pressed", async () => {
    const user = userEvent.setup();
    render(<PowerMenu />);
    await waitFor(() => screen.getByRole("button", { name: "Power menu" }));
    await user.click(screen.getByRole("button", { name: "Power menu" }));
    expect(screen.getByRole("menu")).toBeTruthy();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("calls power_action when a menu item is clicked", async () => {
    const user = userEvent.setup();
    render(<PowerMenu />);
    await waitFor(() => screen.getByRole("button", { name: "Power menu" }));
    await user.click(screen.getByRole("button", { name: "Power menu" }));
    await user.click(screen.getByRole("menuitem", { name: /Reboot/ }));
    expect(invoke).toHaveBeenCalledWith("power_action", { action: "reboot" });
  });

  it("closes the dropdown after a power action is selected", async () => {
    const user = userEvent.setup();
    render(<PowerMenu />);
    await waitFor(() => screen.getByRole("button", { name: "Power menu" }));
    await user.click(screen.getByRole("button", { name: "Power menu" }));
    await user.click(screen.getByRole("menuitem", { name: /Suspend/ }));
    // Menu should close immediately (setOpen(false) is called before invoke).
    expect(screen.queryByRole("menu")).toBeNull();
  });
});
