//! Defines bevy Components used by the particle system.

use bevy::prelude::{
    Bundle, Entity, GlobalTransform, Resource, Transform, Vec3, VisibilityBundle,
    {Component, Handle, Image},
};

use crate::values::{ColorOverTime, JitteredValue, ValueOverTime};

/// Defines a burst of a specified number of particles at the given time in a running particle system.
///
/// Bursts do not count as part of the per-second spawn rate.
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
pub enum ParticleSpace {
    /// Indicates particles should move relative to a parent.
    Local,
    /// Indicates particles should move relative to the world.
    World,
}

/// Defines the parameters of how a system and its particles behave.
///
/// A [`ParticleSystem`] will emit particles until it reaches the ``system_duration_seconds`` or forever if ``looping`` is true, so long as the
/// entity with the [`ParticleSystem`] also has a [`Playing`] component.
///
/// If a [`ParticleSystem`] component is removed before all particles have finished their lifetime, the associated particles will all despawn themselves
/// on the next frame.
#[derive(Debug, Component, Clone)]
pub struct ParticleSystem {
    /// The maximum number of particles the system can have alive at any given time.
    pub max_particles: usize,

    /// The sprite used for each particle.
    pub default_sprite: Handle<Image>,

    /// The number of particles to spawn per second.
    ///
    /// This uses a [`ValueOverTime`] so that the spawn rate can vary over the lifetime of the system.
    pub spawn_rate_per_second: ValueOverTime,

    /// The raidus around the particle systems location that particles will spawn in.
    ///
    /// Setting this to zero will make all particles start at the same position.
    /// Setting this to a non-jittered constant will make particles spawn exactly that distance away from the
    /// center position. Jitter will allow particles to spawn in a range.
    pub spawn_radius: JitteredValue,

    /// The shape of the emitter, defined in radian.
    ///
    /// The default is [`std::f32::consts::TAU`], which results particles going in all directions in a circle.
    /// Reducing the value reduces the possible emitting directions. [`std::f32::consts::PI`] will emit particles
    /// in a semi-circle.
    pub emitter_shape: f32,

    /// The rotation angle of the emitter, defined in radian.
    ///
    /// Zero indicates straight up in the Y direction. [`std::f32::consts::PI`] indicates straight down in the Y direction.
    pub emitter_angle: f32,

    /// The initial movement velocity of a particle.
    ///
    /// This value can be constant, or have added jitter to have particles move at varying speeds.
    pub initial_velocity: JitteredValue,

    /// The acceleration of each particle.
    ///
    /// This value can change over time. Zero makes the particle move at its ``initial_velocity`` for its lifetime.
    pub acceleration: ValueOverTime,

    /// The lifetime of each particle, in seconds.
    ///
    /// This value can have jitter, causing lifetimes to vary per particle.
    pub lifetime: JitteredValue,

    /// The color of each particle over time.
    ///
    /// Color is used to modify the ``default_sprite``. A constant value of [`bevy::prelude::Color::WHITE`] will make the sprite appear with no modifications.
    ///
    /// This can vary over time and be used to modify alpha as well.
    pub color: ColorOverTime,

    /// The scale or size of the particle over time.
    ///
    /// Changing this value over time shrinks or grows the particle accordingly.
    pub scale: ValueOverTime,

    /// Whether or not the system will start over automatically.
    pub looping: bool,

    /// How long the system will emit particles for.
    pub system_duration_seconds: f32,

    /// A maximum distance a particle can travel before being despawend.
    pub max_distance: Option<f32>,

    /// Set a fixed/constant z value (useful for 2D to set a fixed z-depth).
    pub z_value_override: Option<JitteredValue>,

    /// A series of bursts of particles at configured times.
    pub bursts: Vec<ParticleBurst>,

    /// What coordinate space particles should use.
    pub space: ParticleSpace,

    /// `true` indicates that the system will use scaled time if it is present, `false` will result in always using real time.
    pub use_scaled_time: bool,

    /// Indicates that the entity the [`ParticleSystem`] is on should be despawned when the system completes and has no more particles.
    ///
    /// Defaults to `false`.
    ///
    /// Note that this will never trigger on a system that has ``looping`` set to `true`.
    pub despawn_on_finish: bool,
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self {
            max_particles: 100,
            default_sprite: Handle::default(),
            spawn_rate_per_second: 5.0.into(),
            spawn_radius: 0.0.into(),
            emitter_shape: std::f32::consts::TAU,
            emitter_angle: 0.0,
            initial_velocity: 1.0.into(),
            acceleration: 0.0.into(),
            lifetime: 5.0.into(),
            color: ColorOverTime::default(),
            scale: 1.0.into(),
            looping: true,
            system_duration_seconds: 5.0,
            max_distance: None,
            z_value_override: None,
            bursts: Vec::default(),
            space: ParticleSpace::World,
            use_scaled_time: true,
            despawn_on_finish: false,
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
/// The parent should be linked here explicitly because particles may oprate in world space, and not be actual
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
    /// When the [`DistanceTraveled`] component value reaches this value, the particle is considered dead and will be despawned.
    pub max_distance: Option<f32>,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            parent_system: Entity::from_raw(0),
            max_lifetime: f32::default(),
            max_distance: None,
        }
    }
}

/// Contains how long a particle has been alive, in seconds.
#[derive(Debug, Component, Default)]
pub struct Lifetime(pub f32);

/// Contains how far, in world units, a particle has moved since spawning.
#[derive(Debug, Component, Default)]
pub struct DistanceTraveled(pub f32);

/// Defines the current velocity of an individual particle entity.
#[derive(Debug, Component, Default)]
pub struct Velocity(pub f32);

/// Defines the direction a particle is currently traveling.
#[derive(Debug, Component, Default)]
pub struct Direction(pub Vec3);

impl Direction {
    /// Creates a new [`Direction`] based on a [`Vec3`].
    ///
    /// ``ignore_z`` should generally be set to true for 2d use cases, so trajectories ignore the z dimension and a particle stays at a consistent depth.
    pub fn new(mut direction: Vec3, ignore_z: bool) -> Self {
        if ignore_z {
            direction.z = 0.0;
        }
        Self(direction.normalize())
    }
}

/// Marker component indicating that the [`ParticleSystem`] on the same entity is currently Playing.
#[derive(Debug, Component)]
pub struct Playing;

/// Tracks running state of the [`ParticleSystem`] on the same entity.
#[derive(Debug, Component, Default)]
pub struct RunningState {
    /// Tracks the current amount of time since the start of the system.
    ///
    /// This is reset when the running time surpases the ``system_duration_seconds``.
    pub running_time: f32,

    /// The truncated current second.
    pub current_second: f32,

    /// The number of particles already spawned during ``current_second``.
    ///
    /// This number is reset when ``current_second`` rolls over.
    pub spawned_this_second: usize,
}

/// Tracks the current particle count for the [`ParticleSystem`] on the same entity.
#[derive(Debug, Component, Default)]
pub struct ParticleCount(pub usize);

/// Tracks the current index for particle bursts for the [`ParticleSystem`] on the same entity.
#[derive(Debug, Component, Default)]
pub struct BurstIndex(pub usize);

/// A spawnable bundle for a [`ParticleSystem`] containing all of the necssary components.
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

#[derive(Debug, Bundle, Default)]
pub(crate) struct ParticleBundle {
    pub particle: Particle,
    pub lifetime: Lifetime,
    pub velocity: Velocity,
    pub direction: Direction,
    pub distance: DistanceTraveled,
}

/// Specifies the time scaling for all particle systems.
///
/// This can be used to speed up the particle system if the speed of time of the game changes.
/// The contained value is used as a multiplier for the frame delta time.
#[derive(Debug, Clone, Copy, Resource)]
pub struct TimeScale(pub f32);

impl Default for TimeScale {
    fn default() -> Self {
        Self(1.0)
    }
}
