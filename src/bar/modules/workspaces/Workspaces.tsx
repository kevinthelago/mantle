import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Workspace, WorkspaceState } from "../../../types/config";
import "./workspaces.css";

function outputFromLabel(label: string): string {
  return label.replace(/^bar-/, "") || "*";
}

/**
 * Workspaces module.
 *
 * Subscribes to `workspace-state` events emitted by the compositor stream's
 * Rust backend.  Clicking a workspace button dispatches via the compositor.
 */
export default function Workspaces() {
  const outputName = outputFromLabel(getCurrentWindow().label);
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);

  useEffect(() => {
    // Request current state from the compositor service.
    invoke<WorkspaceState>("get_workspace_state")
      .then((state) => setWorkspaces(filterForOutput(state.workspaces, outputName)))
      .catch(console.error);

    const unlisten = listen<WorkspaceState>("workspace-state", (event) => {
      setWorkspaces(filterForOutput(event.payload.workspaces, outputName));
    });

    return () => { unlisten.then((f) => f()); };
  }, [outputName]);

  const activate = (name: string) => {
    invoke("compositor_dispatch", { command: `workspace ${name}` }).catch(console.error);
  };

  return (
    <div className="bar-module workspaces" role="tablist" aria-label="Workspaces">
      {workspaces.map((ws) => (
        <button
          key={ws.id}
          role="tab"
          aria-selected={ws.focused}
          className={[
            "workspace-btn",
            ws.focused && "workspace-btn--focused",
            ws.urgent && "workspace-btn--urgent",
          ]
            .filter(Boolean)
            .join(" ")}
          onClick={() => activate(ws.name)}
        >
          {ws.name}
        </button>
      ))}
    </div>
  );
}

function filterForOutput(workspaces: Workspace[], output: string): Workspace[] {
  if (output === "*") return workspaces;
  return workspaces.filter((ws) => ws.output === output || ws.output === "");
}
