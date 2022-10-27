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
//!     .spawn(ParticleSystemBundle {
//!         particle_system: ParticleSystem {
//!             max_particles: 10_000,
//!             default_sprite: asset_server.load("my_particle.png"),
//!             spawn_rate_per_second: 25.0.into(),
//!             initial_velocity: JitteredValue::jittered(3.0, -1.0..1.0),
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

use bevy::prelude::{App, Plugin, *};
pub use components::*;
use systems::{
    partcle_spawner, particle_cleanup, particle_color, particle_lifetime, particle_transform,
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
        app.add_system(partcle_spawner)
            .add_system(particle_lifetime)
            .add_system(particle_color)
            .add_system(particle_transform)
            .add_system(particle_cleanup)
            .init_resource::<TimeScale>();
    }
}
