# bevy_simplesave

[![Crates.io](https://img.shields.io/crates/v/bevy_simplesave.svg)](https://crates.io/crates/bevy_simplesave)

A small, dependency-light crate for saving Bevy `Resource`s to `.ron` files.

## How to use

1. Define a group with a marker type that implements `SaveGroup`.
2. Derive `SaveMember` on each resource and give it a key with `#[save(key = "...")]`.
3. Register the members with `register_group_member`. This does not create the resources.
4. Save with `save_group` and load with `load_group`.

No plugin is needed.

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

A key must be unique within a group (registering two types under one key panics) and must not change once files exist.
`SaveMember` can also be implemented by hand: `impl SaveMember for Health { const KEY: &'static str = "health"; }`.

### Reading one member

`read_member` reads a single member without a world or a registration, e.g. to list save slots:

```rust
use bevy_simplesave::read_member;

match read_member::<Health>("saves/slot_1.ron") {
    Ok(health) => { /* usable slot */ }
    Err(e) if e.is_not_found() => { /* empty slot */ }
    Err(e) => { /* broken slot */ }
}
```

It also suits settings, where a missing or broken value should become a default:
`read_member::<Settings>(path).unwrap_or_default()`.

### Where to put the file

The functions take any path. `SaveLocation` (`ExeRelative`, `AppData` or `Custom`) resolves to one:

```rust
use bevy_simplesave::SaveLocation;

let path = SaveLocation::ExeRelative("saves/slot_1.ron".into()).resolve()?;
```

### Errors

`save_group` returns a `SaveWriteError`; `load_group` and `read_member` return a `SaveReadError`. Nothing panics except
registering two types under one key.

## What it does and does not do

- Saves/loads plain `Resource` structs as `.ron` files, bundled by group.
- Writes files atomically (temporary file, then rename).
- Runs on Windows / Linux / macOS (not WASM).
- No encryption / obfuscation.
- Missing fields are filled in via `#[serde(default)]`: perform a schema migration yourself if necessary.

## Compatible Bevy versions

| Bevy version | `bevy_simplesave` version |
|:-------------|:--------------------------|
| `0.20`       | `0.2`                     |

## License

MIT or Apache-2.0
