#![warn(future_incompatible, missing_docs, clippy::pedantic)]
#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::needless_pass_by_value,
    clippy::type_complexity
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
//!         .add_plugins(ParticleSystemPlugin::default()) // <-- Add the plugin
//!         // ...
//!         .add_systems(Startup, spawn_particle_system)
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
//!             texture: ParticleTexture::Sprite(asset_server.load("px.png")),
//!             spawn_rate_per_second: 25.0.into(),
//!             initial_speed: JitteredValue::jittered(3.0, -1.0..1.0),
//!             lifetime: JitteredValue::jittered(8.0, -2.0..2.0),
//!             color: ColorOverTime::Gradient(Curve::new(vec![
//!                 CurvePoint::new(Color::WHITE, 0.0),
//!                 CurvePoint::new(Color::rgba(0.0, 0.0, 1.0, 0.0), 1.0),
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

use bevy_app::{
    prelude::{App, Plugin},
    Update,
};
use bevy_ecs::prelude::IntoSystemConfigs;
use bevy_math::Vec3;
use bevy_reflect::std_traits::ReflectDefault;
use bevy_render::color::Color;
pub use components::*;
pub use systems::ParticleSystemSet;
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
///     .add_plugins((DefaultPlugins, ParticleSystemPlugin::default())) // <-- Add the plugin
///     // ...
///     .run();
/// }
/// ```
#[derive(Default)]
pub struct ParticleSystemPlugin;

impl Plugin for ParticleSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                particle_spawner,
                particle_lifetime,
                particle_sprite_color,
                particle_texture_atlas_color,
                particle_transform,
                particle_cleanup,
            )
                .into_configs()
                .in_set(ParticleSystemSet),
        );
        app.register_type::<Curve<f32>>()
            .register_type::<Curve<Vec3>>()
            .register_type::<Curve<Color>>()
            .register_type::<Lerp<f32>>()
            .register_type_data::<Lerp<f32>, ReflectDefault>()
            .register_type::<Lerp<Vec3>>()
            .register_type_data::<Lerp<Vec3>, ReflectDefault>()
            .register_type::<Lerp<Color>>()
            .register_type_data::<Lerp<Color>, ReflectDefault>()
            .register_type::<ValueOverTime>()
            .register_type::<VectorOverTime>()
            .register_type::<ColorOverTime>()
            .register_type::<VelocityModifier>()
            .register_type::<Noise2D>()
            .register_type::<SinWave>()
            .register_type::<ParticleSystem>()
            .register_type::<ParticleCount>()
            .register_type::<RunningState>()
            .register_type::<BurstIndex>();
    }
}
