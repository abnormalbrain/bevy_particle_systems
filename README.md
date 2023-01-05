# bevy_particle_systems

---
[![Crates.io](https://img.shields.io/crates/v/bevy_particle_systems)](https://crates.io/crates/bevy_particle_systems)
[![docs](https://docs.rs/bevy_particle_systems/badge.svg)](https://docs.rs/bevy_particle_systems/)
[![MIT](https://img.shields.io/crates/l/bevy_particle_systems)](./LICENSE)

A particle system plugin for [bevy](https://bevyengine.org)

Currently sprite based and focused on 2D.

## Example

![](https://github.com/abnormalbrain/bevy_particle_systems/blob/main/assets/example.gif)
 
The above was captured running a release build of the `basic` example, `cargo run --example basic --release`, and ran at 140-150 FPS on a
2019 Intel i9 MacBook Pro, rendering about 10k particles.

```
INFO bevy diagnostic: frame_time                      :    0.006995s (avg 0.006966s)
INFO bevy diagnostic: fps                             :  142.956068  (avg 143.654887)
INFO bevy diagnostic: entity_count                    : 10132.000000  (avg 10149.350000)
```

## Usage

1. Add the [`ParticleSystemPlugin`] plugin.

```rust
use bevy::prelude::*;
use bevy_particle_systems::ParticleSystemPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ParticleSystemPlugin::default()) // <-- Add the plugin
        // ...
        .add_startup_system(spawn_particle_system)
        .run();
}

fn spawn_particle_system() { /* ... */ }
```

2. Spawn a particle system whenever necessary.
```rust
use bevy::prelude::*;
use bevy_particle_systems::*;

fn spawn_particle_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
    // Add the bundle specifying the particle system itself.
    .spawn(ParticleSystemBundle {
        particle_system: ParticleSystem {
            max_particles: 10_000,
            default_sprite: ParticuleTexture::Sprite(asset_server.load("my_particle.png")),
            spawn_rate_per_second: 25.0.into(),
            initial_speed: JitteredValue::jittered(3.0, -1.0..1.0),
            lifetime: JitteredValue::jittered(8.0, -2.0..2.0),
            color: ColorOverTime::Gradient(Gradient::new(vec![
                ColorPoint::new(Color::WHITE, 0.0),
                ColorPoint::new(Color::rgba(0.0, 0.0, 1.0, 0.0), 1.0),
            ])),
            looping: true,
            system_duration_seconds: 10.0,
            ..ParticleSystem::default()
        },
        ..ParticleSystemBundle::default()
    })
    // Add the playing component so it starts playing. This can be added later as well.
    .insert(Playing);
}
```

## Bevy Versions

|`bevy_particle_systems`|`bevy`|
|:--|:--|
|0.6|0.9|
|0.5|0.8|
|0.4|0.7|
