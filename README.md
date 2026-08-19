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

```rust
use bevy::prelude::*;
use bevy_simplesave::{SaveGroup, SaveGroupExt, SimpleSavePlugin, save_group, load_group};
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
        .add_systems(Update, your_system)
        .run();
}

fn your_system(keys: Res<ButtonInput<KeyCode>>, world: &World) {
    if keys.just_pressed(KeyCode::F5) {
        let slot = 1;
        save_group::<SlotGroup>(world, format!("saves/slot_{slot}.ron"))
            .expect("failed to save slot");
    }
}
```

## What it does and does not do

- Saves/loads plain `Resource` structs as `.ron` files (see above for locations, timing, and grouping).
- Runs on Windows / Linux / macOS (not WASM).
- No encryption / obfuscation.
- Missing values are filled in via `#[serde(default)]`: perform a schema migration yourself if necessary.

## License

MIT or Apache-2.0
