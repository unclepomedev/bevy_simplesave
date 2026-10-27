# bevy_simplesave

[![Crates.io](https://img.shields.io/crates/v/bevy_simplesave.svg)](https://crates.io/crates/bevy_simplesave)

A small, dependency-light crate for saving Bevy `Resource`s to `.ron` files.

## How to use

### Saving a single resource

1. (must) Add `SimpleSavePlugin` to your app.
2. Derive `SaveResource` on your resource.
3. Add the timing to save your resource with `#[save(timing = auto | manual)]`.
    * `auto`: saves the resource whenever it changes
    * `manual`: saves the resource when `save_now` is called
4. Register your resource with `register_saveable`.
    * The `SaveLocation`s are `ExeRelative`, `AppData` and `Custom`.

```rust
use bevy::prelude::*;
use bevy_simplesave::{SaveAppExt, SaveLocation, SimpleSavePlugin, SaveResource};
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Default, SaveResource)]  // 2
#[save(timing = auto)]  // 3
struct Settings { volume: f32 }

fn main() {
    App::new()
        .add_plugins(SimpleSavePlugin)  // 1
        .register_saveable::<Settings>(SaveLocation::ExeRelative("data/settings.ron".into()))  // 4
        .run();
}
```

### Bundling multiple resources into one file

Use `register_group_member` instead of `register_saveable` when several
resources should be saved and loaded together as a single file (e.g. save slots).
Registering a member inserts its `Default` value into the world right away.

```rust
use bevy::prelude::*;
use bevy_simplesave::{SaveGroup, SaveGroupExt, SimpleSavePlugin, load_group_strict, save_group};
use serde::{Deserialize, Serialize};

struct SlotGroup;
impl SaveGroup for SlotGroup {}

#[derive(Resource, Serialize, Deserialize, Default)]
struct Position { x: f32, y: f32 }

#[derive(Resource, Serialize, Deserialize, Default)]
struct Health { hp: u32 }

fn main() {
    App::new()
        .add_plugins(SimpleSavePlugin)
        .register_group_member::<SlotGroup, Position>()
        .register_group_member::<SlotGroup, Health>()
        .add_systems(Update, (save_slot, load_slot))
        .run();
}

fn save_slot(world: &World) {
    save_group::<SlotGroup>(world, "saves/slot_1.ron").expect("failed to save slot");
}

// Loading needs `&mut World` (an exclusive system).
fn load_slot(world: &mut World) {
    if let Err(e) = load_group_strict::<SlotGroup>(world, "saves/slot_1.ron") {
        error!("failed to load slot: {e}");
    }
}
```

There are two ways to load a group:

| Function            | Missing file                | Missing / broken member                                | On failure                  |
|:--------------------|:----------------------------|:-------------------------------------------------------|:----------------------------|
| `load_group`        | every member gets `Default` | that member gets `Default` and `LoadFailed` is written | still returns `Ok`          |
| `load_group_strict` | `Err` (`is_not_found()`)    | `Err`                                                  | the world is left untouched |

Use `load_group_strict` when a broken file must not turn into silent defaults (e.g. save slots).

To inspect a file without touching the world (e.g. to list save slots), read a single member:

```rust
use bevy_simplesave::read_member;

match read_member::<Health>("saves/slot_1.ron") {
    Ok(health) => { /* usable slot */ }
    Err(e) if e.is_not_found() => { /* empty slot */ }
    Err(e) => { /* broken slot */ }
}
```

### Handling errors

Save/load failures inside systems never panic. Instead, the plugin writes a `SaveFailed` or `LoadFailed` message,
which you can read with a `MessageReader` to log or notify the user:

```rust
use bevy::prelude::*;
use bevy_simplesave::{LoadFailed, SaveFailed};

fn report_errors(mut saves: MessageReader<SaveFailed>, mut loads: MessageReader<LoadFailed>) {
    for msg in saves.read() {
        error!("failed to save {}: {}", msg.resource_type, msg.error);
    }
    for msg in loads.read() {
        error!("failed to load {}: {}", msg.resource_type, msg.error);
    }
}
```

`LoadFailed` is written when a registered resource or a group member (via `load_group`) falls back to its default.
`save_group`, `load_group_strict` and `read_member` return a `Result` instead.

## What it does and does not do

- Saves/loads plain `Resource` structs as `.ron` files (see above for locations, timing, and grouping).
- Writes files atomically (temporary file, then rename).
- Runs on Windows / Linux / macOS (not WASM).
- No encryption / obfuscation.
- Missing fields are filled in via `#[serde(default)]`: perform a schema migration yourself if necessary.
- Group members are stored under their Rust type name (`std::any::type_name`):
  renaming or moving a type makes its old data unreadable.

## Compatible Bevy versions

| Bevy version | `bevy_simplesave` version |
|:-------------|:--------------------------|
| `0.20`       | `0.1`                     |

## License

MIT or Apache-2.0
