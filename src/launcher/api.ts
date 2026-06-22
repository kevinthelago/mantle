import { invoke } from "@tauri-apps/api/core";
import type { SearchResult } from "./types";

export const api = {
  search(query: string): Promise<SearchResult[]> {
    return invoke("search_apps", { query });
  },

  launch(id: string): Promise<void> {
    return invoke("launch_app", { id });
  },

  runCommand(cmd: string): Promise<void> {
    return invoke("run_command", { cmd });
  },

  close(): Promise<void> {
    return invoke("close_launcher");
  },
};
