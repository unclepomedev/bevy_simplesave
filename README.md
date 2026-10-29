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

If the file is missing or broken, the resource falls back to `Default` (and a `LoadFailed` message is written for a broken file).

### Bundling multiple resources into one file

Use a save group when several resources should be saved and loaded together as a single file (e.g. save slots).
Group files are read strictly: nothing silently falls back to a default.

1. Define a group with a marker type that implements `SaveGroup`.
2. Derive `SaveMember` on each resource and give it a key with `#[save(key = "...")]`.
3. Register the members with `register_group_member`. This does not create the resources.
4. Save with `save_group` and load with `load_group`.

```rust
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_simplesave::{SaveGroup, SaveGroupExt, SaveMember, load_group, save_group};
use serde::{Deserialize, Serialize};

struct SlotGroup;  // 1
impl SaveGroup for SlotGroup {}

#[derive(Resource, Serialize, Deserialize, SaveMember)]  // 2
#[save(key = "position")]
struct Position { x: f32, y: f32 }

#[derive(Resource, Serialize, Deserialize, SaveMember)]
#[save(key = "health")]
struct Health { hp: u32 }

fn main() {
    App::new()
        .register_group_member::<SlotGroup, Position>()  // 3
        .register_group_member::<SlotGroup, Health>()
        .insert_resource(Position { x: 0.0, y: 0.0 })
        .insert_resource(Health { hp: 100 })
        .add_systems(
            Update,
            (
                save_slot.run_if(input_just_pressed(KeyCode::F5)),
                load_slot.run_if(input_just_pressed(KeyCode::F9)),
            ),
        )
        .run();
}

fn save_slot(world: &World) {  // 4
    if let Err(e) = save_group::<SlotGroup>(world, "saves/slot_1.ron") {
        error!("failed to save slot: {e}");
    }
}

// Loading needs `&mut World` (an exclusive system).
fn load_slot(world: &mut World) {
    if let Err(e) = load_group::<SlotGroup>(world, "saves/slot_1.ron") {
        error!("failed to load slot: {e}");
    }
}
```

`load_group` fails if the file or any member is missing or broken. On failure the world is left untouched.
`SaveReadError::is_not_found()` tells a missing file from a broken one.

To inspect a file without touching the world (e.g. to list save slots), read a single member:

```rust
use bevy_simplesave::read_member;

match read_member::<Health>("saves/slot_1.ron") {
    Ok(health) => { /* usable slot */ }
    Err(e) if e.is_not_found() => { /* empty slot */ }
    Err(e) => { /* broken slot */ }
}
```

`SaveMember` can also be implemented by hand: `impl SaveMember for Health { const KEY: &'static str = "health"; }`.
A key must be unique within a group (registering two types under one key panics) and must not change once files exist.

### Handling errors

Save/load failures of single resources inside systems never panic. Instead, the plugin writes a `SaveFailed` or
`LoadFailed` message, which you can read with a `MessageReader` to log or notify the user:

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

The group functions (`save_group`, `load_group`, `read_member`) return a `Result` instead.

## What it does and does not do

- Saves/loads plain `Resource` structs as `.ron` files (see above for locations, timing, and grouping).
- Writes files atomically (temporary file, then rename).
- Runs on Windows / Linux / macOS (not WASM).
- No encryption / obfuscation.
- Missing fields are filled in via `#[serde(default)]`: perform a schema migration yourself if necessary.
- A type cannot be both a single saved resource (`SaveResource`) and a group member (`SaveMember`).

## Compatible Bevy versions

| Bevy version | `bevy_simplesave` version |
|:-------------|:--------------------------|
| `0.20`       | `0.1`                     |

## License

MIT or Apache-2.0
