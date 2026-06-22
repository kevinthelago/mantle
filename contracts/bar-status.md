# bar-status stream — integration contract

## Tauri commands exposed (register in lib.rs)

```rust
// power_supply
use crate::services::power_supply::{get_battery_snapshot, PowerSupplyService};

// network
use crate::services::network::{get_network_snapshot, NetworkService};

// audio
use crate::services::audio::{get_audio_snapshot, set_audio_volume, set_audio_mute, AudioService};

// brightness
use crate::services::brightness::{get_brightness_snapshot, set_brightness, BrightnessService};

// tray
use crate::tray::{get_tray_items, tray_item_activate, tray_menu_event, tray_refresh_menu, TrayService};
```

### In tauri::Builder setup (lib.rs):

```rust
.manage(PowerSupplyService::init(app.handle().clone()).await)
.manage(NetworkService::init(app.handle().clone()).await)
.manage(AudioService::init(app.handle().clone()))   // sync init, PW thread spawned inside
.manage(BrightnessService::init(app.handle().clone()).await)
.manage(TrayService::init(app.handle().clone()).await)
.invoke_handler(tauri::generate_handler![
    get_battery_snapshot,
    get_network_snapshot,
    get_audio_snapshot,
    set_audio_volume,
    set_audio_mute,
    get_brightness_snapshot,
    set_brightness,
    get_tray_items,
    tray_item_activate,
    tray_menu_event,
    tray_refresh_menu,
])
```

### In Cargo.toml (add to [dependencies]):

```toml
zbus        = "4"
futures-util = "0.3"
pipewire    = "0.8"
base64      = "0.22"
specta      = { version = "2", features = ["derive"] }
log         = "0.4"
```

## Events emitted (frontend listeners)

| Event               | Payload type         | Trigger                             |
| ------------------- | -------------------- | ----------------------------------- |
| `battery_update`    | `BatterySnapshot`    | UPower property change              |
| `network_update`    | `NetworkSnapshot`    | NM state/primary-connection change  |
| `audio_update`      | `AudioSnapshot`      | PipeWire node props / set command   |
| `brightness_update` | `BrightnessSnapshot` | sysfs poll (500 ms)                 |
| `tray_update`       | `TraySnapshot`       | SNI item add/remove/property change |
| `tray_item_removed` | `string` (key)       | SNI item service disappears         |

## Bar-shell module registrations

Each module exports a `barModule` object:

```typescript
{ id: string, region: 'left' | 'center' | 'right', order: number, component: React.ComponentType }
```

Import and register in `src/bar/registry.ts`:

```typescript
import { barModule as battery } from './modules/battery'
import { barModule as network } from './modules/network'
import { barModule as audio } from './modules/audio'
import { barModule as brightness } from './modules/brightness'
import { barModule as tray } from './modules/tray'
;[battery, network, audio, brightness, tray].forEach(registerModule)
```

Default ordering (right region, left-to-right): tray(1), audio(10), brightness(15), network(20), battery(30).

## Graceful degrade

Every Rust service marks `available: false` on its snapshot when the backing
D-Bus service (UPower, NetworkManager, logind, PipeWire, or the session bus)
is absent or returns an error. The React components return `null` when
`available === false`, so they silently disappear from the bar rather than
showing broken UI.
