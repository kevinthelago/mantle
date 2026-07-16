# mantle

![License](https://img.shields.io/github/license/kevinthelago/mantle) ![Last commit](https://img.shields.io/github/last-commit/kevinthelago/mantle)

# Goal

## Overview

# Goal

## Tech stack

# Stack

All versions verified against upstream docs (2026-06). The central decision — **how the webview becomes a layer-shell surface** — is resolved by the day-1 spike: **Path A preferred, Path B fallback.**

## The layer-shell binding decision (verified)

- **Tauri 2.0** ships its webview via **wry → webkit2gtk-4.1**, which is **GTK3**-based (confirmed: Tauri prerequisites pin `webkit2gtk-4.1` + `libappindicator-gtk3`).
- The matching binding is the **`gtk-layer-shell` crate `0.8.x`**, which binds **`gtk` 0.18 (GTK3)** via `gtk-layer-shell-sys 0.7` over the `gtk-layer-shell` C library. **Not** `gtk4-layer-shell` — that pairs with webkitgtk-6.0/GTK4, which Tauri does not use.
- **Path A (preferred):** Tauri 2 app; grab the window's underlying `gtk::ApplicationWindow` (`WebviewWindow::gtk_window()` on Linux) and apply `gtk-layer-shell` to it **before the window is mapped** (init layer, anchors, exclusive zone, layer = Top/Overlay). The risk is purely timing — the window must become a layer surface before it realizes as a normal `xdg-toplevel`.
- **Path B (fallback):** drop Tauri and go `gtk-rs` (`gtk` 0.18) + `webkit2gtk` (`webkit2gtk` 2.0 crate, GTK3) + `gtk-layer-shell` directly — the eww/ags architecture. Choose this only if Tauri's window lifecycle fights the pre-map layer-shell init in the spike.

## Layers

| Layer                  | Choice                                                                   | Version                   | Notes                                                             |
| ---------------------- | ------------------------------------------------------------------------ | ------------------------- | ----------------------------------------------------------------- |
| Core language          | Rust (stable, edition 2021)                                              | latest stable             | the native core                                                   |
| Webview host           | **Tauri 2** (`tauri`, `wry`)                                             | `2.x`                     | Path A; webkit2gtk-4.1 / GTK3                                     |
| Layer shell            | `gtk-layer-shell` + `gtk`                                                | `0.8.x` / `0.18`          | GTK3 binding; C lib `gtk-layer-shell`                             |
| sway IPC               | `swayipc` (tokio)                                                        | latest                    | workspaces/outputs/window events + dispatch                       |
| Hyprland IPC           | `hyprland` crate (or hand-rolled socket2)                                | latest                    | abstracted behind the same bridge trait                           |
| D-Bus                  | `zbus`                                                                   | `4.x`                     | UPower, NetworkManager, `org.freedesktop.Notifications`           |
| Audio                  | **`pipewire` crate** (native PipeWire)                                   | latest                    | volume/sinks/nodes via the native PipeWire API                    |
| UI framework           | **React** + **TypeScript**                                               | React `19`, TS `5.x`      | rendered in the webview                                           |
| Bundler / dev server   | **Vite**                                                                 | `6.x`                     | HMR into the webview via Tauri dev                                |
| Design system          | tokens → primitives → components + **Storybook**                         | SB `8.x`                  | local Claude-style DS                                             |
| Styling                | **CSS + CSS Modules + Sass + Tailwind**, tokens as CSS custom properties | dart-sass, Tailwind `3.x` | all three supported; tokens shared via CSS vars                   |
| Runtime styling loader | **fetch styling toolchains on demand** (Sass/Tailwind/PostCSS)           | —                         | trusted sources only (npm registry, HTTPS + integrity); see below |
| JS package manager     | **npm**                                                                  | `10.x`                    | ships with Node, zero extra setup                                 |
| Build/orchestration    | Tauri CLI (`@tauri-apps/cli`) over cargo + Vite                          | `2.x`                     | one `npm run tauri dev` loop                                      |
| Dev compositor         | **sway**, run nested                                                     | —                         | spike + daily dev target                                          |

## Toolchain binaries

`cargo`, `rustc`, `pnpm`, `node`, `tauri` (via `pnpm tauri`), `storybook`. Recorded in `commands.json` once the repo is linked.

## Styling toolchain

Plain **CSS** and **CSS Modules** are the baseline (zero-runtime, perf-safe for a constantly-repainting shell). **Sass** (dart-sass via Vite) and **Tailwind** (PostCSS, design tokens wired into the config) are both supported on top. Design **tokens live as CSS custom properties** so they're shared identically across CSS/Sass/Tailwind and remain themeable at runtime.

## Runtime styling loader (scoped capability)

Mantle can **fetch styling toolchains on demand at runtime** — Sass, Tailwind, PostCSS plugins — so the UI can adopt new styling tooling **without a full rebuild**. Bounded to _styling_ (not arbitrary UI modules or system packages).

- **Trusted sources only:** a fixed allowlist — the **npm registry** over HTTPS with **integrity (hash) verification**. No arbitrary URLs, no system package manager.
- Open architectural questions (dissected in `architecture` and broken into its own feature in the workshop): where fetched artifacts land on disk, how a freshly-fetched preprocessor is invoked and its output injected into the running webview, cache/versioning, and failure handling when a fetch fails or a build errors.

## Justification for non-obvious picks

- **CSS + Sass + Tailwind with tokens as CSS variables** — supports the author's preferred authoring styles while keeping a single source of truth for tokens; baseline CSS/CSS-Modules path stays zero-runtime for the hot shell surfaces.
- **`swayipc` + `hyprland` behind one trait** — both compositors are primary targets; the bridge exposes a single `CompositorBackend` trait so UI hooks never branch on compositor.
- **Native `pipewire` crate (not the pulse-compat binding)** — chosen for direct, correct control over PipeWire nodes/sinks on modern target systems, accepting the higher complexity.

## Getting started

```bash
git clone https://github.com/kevinthelago/mantle.git
cd mantle
# install dependencies and run the project's build/test/dev commands
```

### Optional: PipeWire audio module

The audio bar module (volume/mute/sink) is built on the native `pipewire` crate
and is gated behind the **non-default** `pipewire-audio` Cargo feature, so a stock
`npm run tauri:build` leaves it out and the `audio` bar module simply renders
nothing. To include it:

```bash
# adds the pipewire dev headers (Debian/Ubuntu)
sudo apt-get install -y libpipewire-0.3-dev
# build with the audio backend enabled
npm run tauri:build:audio
```

Without the feature the rest of the shell is unaffected — every other bar module,
the launcher, widgets, and notifications work as normal.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and our [Code of Conduct](CODE_OF_CONDUCT.md).

## License

See [LICENSE](LICENSE).

---

_Scaffolded by base-studio-code._
