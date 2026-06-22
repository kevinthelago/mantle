# Core Stream — Contracts

These are the interfaces the widgets stream depends on from the core stream.

## LayerShellManager

The core stream is responsible for turning the `widgets` Tauri window into a
`gtk-layer-shell` surface before it maps. The widgets stream does NOT call
`gtk-layer-shell` directly.

### Expected behaviour

- The `widgets` window (label `"widgets"` in `tauri.conf.json`) is claimed by
  `LayerShellManager` during `setup()`.
- Layer: `Top`
- Exclusive zone: `0` (widgets float, they do not reserve screen space)
- Keyboard mode: `OnDemand` — keyboard is grabbed only when the user interacts
  with the surface, and released afterward
- Anchor: configurable (default: none — centred)
- The manager must complete before the window is shown for the first time
  (enforced by starting `visible: false` in `tauri.conf.json`)

### Tauri commands expected

None — the layer-shell setup is handled entirely on the Rust side.

### Events emitted

None — the widgets stream manages its own visibility via `toggle_widget_layer`.

---

## toggle_widget_layer (command)

Provided **by** the widgets stream (`src-tauri/src/lib.rs`).

```
invoke("toggle_widget_layer") -> Result<(), String>
```

Shows the window if hidden; hides it if visible. The core stream's keybind
handler calls this command to toggle the widget overlay.
