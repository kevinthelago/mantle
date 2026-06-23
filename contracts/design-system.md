# Design System Stream — Contracts

The widgets stream currently uses locally-defined CSS Modules for all styling.
Once the design-system stream delivers its token layer, widgets will consume
CSS custom properties via the token contract below.

## CSS custom properties (tokens)

When the design-system stream ships tokens, widgets expect:

| Token                    | Purpose                            |
| ------------------------ | ---------------------------------- |
| `--color-surface`        | Widget background base             |
| `--color-surface-hover`  | Button hover                       |
| `--color-text-primary`   | Primary text                       |
| `--color-text-secondary` | Secondary / muted text             |
| `--color-text-disabled`  | Disabled state                     |
| `--color-accent`         | Highlight / today indicator        |
| `--radius-md`            | Widget corner radius               |
| `--blur-surface`         | `backdrop-filter: blur(...)` value |

Until the design-system stream lands, widgets hard-code compatible values
directly in their CSS Modules.

## Component primitives

The widgets stream does not import any components from the design-system stream
directly. Future candidates for promotion to the design system:

- `<Sparkline>` (currently in `src/widgets/system-monitor/Sparkline.tsx`)
- Error boundary shell (currently `WidgetSlot.tsx`)
