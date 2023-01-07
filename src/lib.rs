#![warn(future_incompatible, missing_docs, clippy::pedantic)]
#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::needless_pass_by_value
)]
//! A particle system plugin for [bevy](https://bevyengine.org)
//!
//! Currently sprite based and focused on 2D.
//!
//! ## Usage
//!
//! 1. Add the [`ParticleSystemPlugin`] plugin.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_particle_systems::ParticleSystemPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(ParticleSystemPlugin::default()) // <-- Add the plugin
//!         // ...
//!         .add_startup_system(spawn_particle_system)
//!         .run();
//! }
//!
//! fn spawn_particle_system() { /* ... */ }
//! ```
//!
//! 2. Spawn a particle system whenever necessary.
//! ```
//! # use bevy::prelude::*;
//! # use bevy_particle_systems::*;
//!
//! fn spawn_particle_system(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     commands
//!     // Add the bundle specifying the particle system itself.
//!     .spawn_bundle(ParticleSystemBundle {
//!         particle_system: ParticleSystem {
//!             max_particles: 10_000,
//!             texture: ParticleTexture::Sprite(asset_server.load("px.png")),
//!             spawn_rate_per_second: 25.0.into(),
//!             initial_speed: JitteredValue::jittered(3.0, -1.0..1.0),
//!             lifetime: JitteredValue::jittered(8.0, -2.0..2.0),
//!             color: ColorOverTime::Gradient(Gradient::new(vec![
//!                 ColorPoint::new(Color::WHITE, 0.0),
//!                 ColorPoint::new(Color::rgba(0.0, 0.0, 1.0, 0.0), 1.0),
//!             ])),
//!             looping: true,
//!             system_duration_seconds: 10.0,
//!             ..ParticleSystem::default()
//!         },
//!         ..ParticleSystemBundle::default()
//!     })
//!     // Add the playing component so it starts playing. This can be added later as well.
//!     .insert(Playing);
//! }
//! ```
//!
pub mod components;
mod systems;
pub mod values;

use bevy_app::prelude::{App, Plugin};
use bevy_ecs::prelude::SystemSet;
pub use components::*;
pub use systems::ParticleSystemLabel;
use systems::{
    particle_cleanup, particle_lifetime, particle_spawner, particle_sprite_color,
    particle_texture_atlas_color, particle_transform,
};
pub use values::*;

/// The plugin component to be added to allow particle systems to run.
///
/// ## Examples
///
/// ```no_run
/// # use bevy::prelude::*;
///
/// use bevy_particle_systems::ParticleSystemPlugin;
///
/// fn main() {
///   App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugin(ParticleSystemPlugin::default()) // <-- Add the plugin
///     // ...
///     .run();
/// }
/// ```
#[derive(Default)]
pub struct ParticleSystemPlugin;

impl Plugin for ParticleSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .label(ParticleSystemLabel::ParticleSystem)
                .with_system(particle_spawner)
                .with_system(particle_lifetime)
                .with_system(particle_sprite_color)
                .with_system(particle_texture_atlas_color)
                .with_system(particle_transform)
                .with_system(particle_cleanup),
        );
        app.register_type::<ParticleSystem>()
            .register_type::<ParticleCount>()
            .register_type::<RunningState>()
            .register_type::<BurstIndex>();
    }
}
