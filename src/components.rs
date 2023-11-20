//! Defines bevy Components used by the particle system.

use bevy_asset::Handle;
use bevy_ecs::prelude::{Bundle, Component, Entity, ReflectComponent};
use bevy_math::{Vec2, Vec3};
use bevy_reflect::prelude::*;
use bevy_render::prelude::{Image, VisibilityBundle};
use bevy_sprite::TextureAtlas;
use bevy_transform::prelude::{GlobalTransform, Transform};

use crate::{
    values::{ColorOverTime, JitteredValue, ValueOverTime},
    AtlasIndex, EmitterShape, VelocityModifier,
};

/// Defines a burst of a specified number of particles at the given time in a running particle system.
///
/// Bursts do not count as part of the per-second spawn rate.
#[derive(Debug, Clone, Copy, Reflect)]
pub struct ParticleBurst {
    /// The time during the life cycle of a system that the burst should occur.
    ///
    /// This value should be strictly less than the particle systems ``system_duration_seconds`` or it will
    /// not fire.
    pub time: f32,

    /// The number of particles to fire at the specified time.
    ///
    /// All particles in a burst are not counted towards the spawn rate, but are counted towards the system maximum.
    /// They follow all other parameters and behaviors of the spawning system.
    pub count: usize,
}

impl ParticleBurst {
    /// Creates a new [`ParticleBurst`] at a specified time of the given number of particles.
    pub fn new(time: f32, count: usize) -> Self {
        Self { time, count }
    }
}

/// Defines what space a particle should operate in.
#[derive(Debug, Clone, Copy, Reflect)]
pub enum ParticleSpace {
    /// Indicates particles should move relative to a parent.
    Local,
    /// Indicates particles should move relative to the world.
    World,
}

/// Defines what texture to use for a particle
#[derive(Debug, Clone, Reflect)]
pub enum ParticleTexture {
    /// Indicates particles should use a given image texture
    Sprite(Handle<Image>),
    /// Indicates particles should use a given texture atlas
    TextureAtlas {
        /// The handle to the texture atlas
        atlas: Handle<TextureAtlas>,
        /// The index in the atlas can constant, or be chosen randomly
        index: AtlasIndex,
    },
}

/// Defines how will be animated the texture atlas index
#[derive(Component, Debug, Clone, Reflect)]
pub struct AnimatedIndex {
    /// At what indices are the different frames on a sprite sheet
    pub indices: Vec<usize>,
    /// How much time each frame should be displayed
    pub time_step: f32,
    /// At what index (from the `indices` field) should start the animation
    pub step_offset: usize,
}

impl AnimatedIndex {
    /// Returns the first index of the animation
    pub fn get_at_start(&self) -> usize {
        let idx = if self.step_offset < self.indices.len() {
            self.step_offset
        } else {
            self.step_offset % self.indices.len()
        };

        self.indices[idx]
    }
    /// Returns the index corresponding at a given time in the animation
    pub fn get_at_time(&self, time: f32) -> usize {
        // Disabling `cast_possible_truncation` so we can truncate the f32.
        // Disabling `pedantic` because we use `abs()` before truncation. No sign loss possible.
        #[allow(clippy::cast_possible_truncation, clippy::pedantic)]
        // take only the integer part.
        let steps_passed = (time / self.time_step).abs() as usize;
        let sample_idx = self.step_offset + steps_passed;

        if sample_idx < self.indices.len() {
            self.indices[sample_idx]
        } else {
            self.indices[sample_idx % self.indices.len()]
        }
    }
}

/// Defines the parameters of how a system and its particles behave.
///
/// A [`ParticleSystem`] will emit particles until it reaches the ``system_duration_seconds`` or forever if ``looping`` is true, so long as the
/// entity with the [`ParticleSystem`] also has a [`Playing`] component.
///
/// If a [`ParticleSystem`] component is removed before all particles have finished their lifetime, the associated particles will all despawn themselves
/// on the next frame.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Component, Clone, Reflect)]
#[reflect(Component)]
pub struct ParticleSystem {
    /// The maximum number of particles the system can have alive at any given time.
    pub max_particles: usize,

    /// The texture used for each particle.
    pub texture: ParticleTexture,

    /// If provided, re-scale the texture size
    ///
    /// This is simply passed directly to `Sprite::custom_size` or `TextureAtlasSprite::custom_size`
    pub rescale_texture: Option<Vec2>,

    /// The number of particles to spawn per second.
    ///
    /// This uses a [`ValueOverTime`] so that the spawn rate can vary over the lifetime of the system.
    pub spawn_rate_per_second: ValueOverTime,

    /// The shape of the emitter.
    pub emitter_shape: EmitterShape,

    /// The initial movement speed of a particle.
    ///
    /// This value can be constant, or have added jitter to have particles move at varying speeds.
    pub initial_speed: JitteredValue,

    /// Modifiers affecting the particle velocity.
    ///
    /// They can be stacked, and will be applied in order.
    /// No modifiers makes the particle move at its ``initial_speed`` for its lifetime.
    pub velocity_modifiers: Vec<VelocityModifier>,

    /// The lifetime of each particle, in seconds.
    ///
    /// This value can have jitter, causing lifetimes to vary per particle.
    pub lifetime: JitteredValue,

    /// The color of each particle over time.
    ///
    /// Color is used to modify the ``default_sprite``. A constant value of [`bevy_render::prelude::Color::WHITE`] will make the sprite appear with no modifications.
    ///
    /// This can vary over time and be used to modify alpha as well.
    pub color: ColorOverTime,

    /// The initial scale of a particle.
    ///
    /// This value can be constant, or have added jitter to have particles with different sizes
    ///
    /// This value is multiplied with scale to produce the final scale.
    pub initial_scale: JitteredValue,

    /// The scale or size of the particle over time.
    ///
    /// Changing this value over time shrinks or grows the particle accordingly.
    pub scale: ValueOverTime,

    /// The rotation of a particle around the `z` access at spawn in radian.
    pub initial_rotation: JitteredValue,

    /// The speed at which the particle rotates in radian per second.
    pub rotation_speed: JitteredValue,

    /// Rotates the particle to be facing the movement direction at spawn.
    ///
    /// This is useful if the image used for the particle has a visual 'forward'
    /// that should match it's movement, such as an arrow.
    ///
    /// This rotation for the movement direction will be added to the `initial_rotation` value,
    /// to account for needing to apply a base rotation to the sprite.
    pub rotate_to_movement_direction: bool,

    /// Whether or not the system will start over automatically.
    pub looping: bool,

    /// How long the system will emit particles for.
    pub system_duration_seconds: f32,

    /// A maximum distance a particle can travel before being despawned.
    pub max_distance: Option<f32>,

    /// Set a fixed/constant z value (useful for 2D to set a fixed z-depth).
    pub z_value_override: Option<JitteredValue>,

    /// A series of bursts of particles at configured times.
    pub bursts: Vec<ParticleBurst>,

    /// What coordinate space particles should use.
    pub space: ParticleSpace,

    /// Dictates whether this system respects Bevy's time scaling by using [`bevy_time::Virtual`]  when true, or [`bevy_time::Real`] when false.
    pub use_scaled_time: bool,

    /// Indicates that the entity the [`ParticleSystem`] is on should be despawned when the system completes and has no more particles.
    ///
    /// Defaults to `false`.
    ///
    /// Note that this will never trigger on a system that has ``looping`` set to `true`.
    pub despawn_on_finish: bool,

    /// Indicates whether alive particles should be despawned when the system itself is despawned.
    ///
    /// When this is `false` (the default), particles will live out their lifetime even if the system has been despawned.
    pub despawn_particles_with_system: bool,
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self {
            max_particles: 100,
            texture: ParticleTexture::Sprite(Handle::default()),
            rescale_texture: None,
            spawn_rate_per_second: 5.0.into(),
            emitter_shape: EmitterShape::default(),
            initial_speed: 1.0.into(),
            velocity_modifiers: vec![],
            lifetime: 5.0.into(),
            color: ColorOverTime::default(),
            initial_scale: 1.0.into(),
            scale: 1.0.into(),
            initial_rotation: 0.0.into(),
            rotation_speed: 0.0.into(),
            rotate_to_movement_direction: false,
            looping: true,
            system_duration_seconds: 5.0,
            max_distance: None,
            z_value_override: None,
            bursts: Vec::default(),
            space: ParticleSpace::World,
            use_scaled_time: true,
            despawn_on_finish: false,
            despawn_particles_with_system: false,
        }
    }
}

impl ParticleSystem {
    /// A oneshot particle system, that doesn't loop and despawns when finished
    ///
    /// Appropriate base for collision effects etc.
    pub fn oneshot() -> Self {
        Self {
            looping: false,
            despawn_on_finish: true,
            ..Default::default()
        }
    }
}

/// An individual Particle, spawned by a [`ParticleSystem`]
///
/// The ``parent_entity`` should link to the entity with the spawning [`ParticleSystem`] on it.
///
/// If the ``parent_entity`` no longer exists or does not contain a [`ParticleSystem`] the particle will
/// be despawned immediately.
///
/// The parent should be linked here explicitly because particles may operate in world space, and not be actual
/// children of the [`ParticleSystem`] itself.
#[derive(Debug, Component)]
pub struct Particle {
    /// The entity on which the spawning [`ParticleSystem`] resides.
    pub parent_system: Entity,

    /// The total lifetime of the particle.
    ///
    /// When the [`Lifetime`] component value reaches this value, the particle is considered dead and will be despawned.
    pub max_lifetime: f32,

    /// The maximum distance traveled for the particle.
    ///
    /// When the [`DistanceTraveled`] component value reaches this value, the particle is considered dead and will be despawned.
    pub max_distance: Option<f32>,

    /// Whether the particle will respect scaled time in its transformations.
    ///
    /// This is copied from [`ParticleSystem::use_scaled_time`] on spawn.
    pub use_scaled_time: bool,

    /// The initial scale of the particle, multiplied with `scale` to produce
    /// the final scale of the particle.
    pub initial_scale: f32,

    /// The scale or size of this particle over time.
    ///
    /// This is copied from [`ParticleSystem::scale`] on spawn.
    pub scale: ValueOverTime,

    /// Velocity Modifiers of this particle.
    ///
    /// This is copied from [`ParticleSystem::velocity_modifiers`] on spawn.
    pub velocity_modifiers: Vec<VelocityModifier>,

    /// The speed, in radian per second, at which the particle rotates.
    ///
    /// This is chosen from [`ParticleSystem::rotation_speed`] on spawn.
    pub rotation_speed: f32,

    /// Indicates whether the particle should be cleaned up when the parent system is despawned
    pub despawn_with_parent: bool,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            parent_system: Entity::from_raw(0),
            max_lifetime: f32::default(),
            max_distance: None,
            use_scaled_time: true,
            initial_scale: 1.0,
            scale: 1.0.into(),
            rotation_speed: 0.0,
            velocity_modifiers: vec![],
            despawn_with_parent: false,
        }
    }
}

/// Holds an individual particles color descriptor.
///
/// This is separated into its own component because the [`ColorOverTime`]
/// is used mutably in the case of Gradients to improve performance.
///
/// Its initial value on particle spawn is copied from [`ParticleSystem::color`]
#[derive(Debug, Component, Default)]
pub struct ParticleColor(pub ColorOverTime);

/// Contains how long a particle has been alive, in seconds.
#[derive(Debug, Component, Default)]
pub struct Lifetime(pub f32);

/// Contains how far, in world units, a particle has moved since spawning.
#[derive(Debug, Component, Default)]
pub struct DistanceTraveled {
    /// The squared distance that the particle has traveled since spawn.
    ///
    /// The squared distance is used instead of the actual distance, since
    /// this is used to compare to the `max_distance` value. When comparing
    /// distances to each other, we can use the `distance_squared` to avoid
    /// a square root, which is computationally very expensive.
    pub dist_squared: f32,
    /// The original spawn point for computing the `dist_squared`
    pub from: Vec3,
}

/// Defines the current velocity of an individual entity particle.
#[derive(Debug, Component, Default)]
pub struct Velocity(pub Vec3);
impl Velocity {
    /// Creates a new [`Velocity`] based on a [`Vec3`].
    ///
    /// ``ignore_z`` should generally be set to true for 2d use cases, so trajectories ignore the z dimension and a particle stays at a consistent depth.
    pub fn new(velocity: Vec3, ignore_z: bool) -> Self {
        if ignore_z {
            Self(Vec3::new(velocity.x, velocity.y, 0.0).normalize() * velocity.length())
        } else {
            Self(velocity)
        }
    }
}

/// Marker component indicating that the [`ParticleSystem`] on the same entity is currently Playing.
#[derive(Debug, Component)]
pub struct Playing;

/// Tracks running state of the [`ParticleSystem`] on the same entity.
#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
pub struct RunningState {
    /// Tracks the current amount of time since the start of the system.
    ///
    /// This is reset when the running time surpasses the ``system_duration_seconds``.
    pub running_time: f32,

    /// The truncated current second.
    pub current_second: f32,

    /// The number of particles already spawned during ``current_second``.
    ///
    /// This number is reset when ``current_second`` rolls over.
    pub spawned_this_second: usize,
}

/// Tracks the current particle count for the [`ParticleSystem`] on the same entity.
#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
pub struct ParticleCount(pub usize);

/// Tracks the current index for particle bursts for the [`ParticleSystem`] on the same entity.
#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
pub struct BurstIndex(pub usize);

/// A spawnable bundle for a [`ParticleSystem`] containing all of the necessary components.
///
/// ``particle_system`` and ``transform`` should generally be the only attributes that need to be overridden.
#[derive(Debug, Default, Bundle)]
pub struct ParticleSystemBundle {
    /// The particle system parameters dictating the spawning and behavior of particles.
    pub particle_system: ParticleSystem,

    /// The location of the [`ParticleSystem`]
    ///
    /// If the particle system is being added to an entity that already has a transform, specify that transform here.
    ///
    /// If the particle system is added as a child to another entity, this will be a relative transform, and will move with the parent entity.
    /// How particles move is independent of this and will be dictated by the particle systems [`ParticleSpace`].
    pub transform: Transform,

    /// The global transform of the particle system.
    ///
    /// This should generally be left at the default.
    pub global_transform: GlobalTransform,

    /// The tracking component for current live particle count.
    ///
    /// This should generally be left at the default.
    pub particle_count: ParticleCount,

    /// The running time tracking component for the particle system.
    ///
    /// This should generally be left at the default.
    pub running_state: RunningState,

    /// The current burst index tracking component.
    ///
    /// This should generally be left at the default.
    pub burst_index: BurstIndex,

    /// Required for child particles to be visible when running in Local space.
    pub visibility: VisibilityBundle,
}

#[derive(Debug, Default, Bundle)]
pub(crate) struct ParticleBundle {
    pub particle: Particle,
    pub lifetime: Lifetime,
    pub velocity: Velocity,
    pub distance: DistanceTraveled,
    pub color: ParticleColor,
}
