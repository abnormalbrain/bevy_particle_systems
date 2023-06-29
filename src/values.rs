//! Different value types and controls used in particle systems.
use std::ops::Range;

use bevy_math::{vec3, Quat, Vec2, Vec3};
use bevy_reflect::{FromReflect, Reflect};
use bevy_render::prelude::Color;
use bevy_sprite::TextureAtlas;
use bevy_transform::prelude::Transform;
use rand::seq::SliceRandom;
use rand::{prelude::ThreadRng, Rng};

use crate::AnimatedIndex;

/// Describes an oriented segment of a circle with a given radius.
#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct CircleSegment {
    /// The shape of the emitter, defined in radians.
    ///
    /// The default is `2 * PI`, which results particles going in all directions in a circle.
    /// Reducing the value reduces the possible emitting directions. [`std::f32::consts::PI`] will emit particles
    /// in a semi-circle.
    pub opening_angle: f32,

    /// The rotation angle of the emitter, defined in radian.
    ///
    /// Zero indicates straight to the right in the X direction. [`std::f32::consts::PI`] indicates straight left in the X direction.
    pub direction_angle: f32,

    /// The radius around the particle systems location that particles will spawn in.
    ///
    /// Setting this to zero will make all particles start at the same position.
    /// Setting this to a non-jittered constant will make particles spawn exactly that distance away from the
    /// center position. Jitter will allow particles to spawn in a range.
    pub radius: JitteredValue,
}

impl Default for CircleSegment {
    fn default() -> Self {
        Self {
            opening_angle: std::f32::consts::TAU,
            direction_angle: 0.0,
            radius: 0.0.into(),
        }
    }
}

impl From<CircleSegment> for EmitterShape {
    fn from(segment: CircleSegment) -> EmitterShape {
        EmitterShape::CircleSegment(segment)
    }
}

/// Defines a line along which particles will be spawned.
#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct Line {
    /// The lenth of the line
    pub length: f32,

    /// The rotation angle of the emitter, defined in radian.
    ///
    /// Zero indicates straight to the right in the +X direction. [`std::f32::consts::PI`] indicates straight left in the -X direction.
    pub angle: JitteredValue,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            length: 1.0,
            angle: 0.0.into(),
        }
    }
}

impl From<Line> for EmitterShape {
    fn from(line: Line) -> EmitterShape {
        EmitterShape::Line(line)
    }
}

/// Defines a sphere within which particles will be spawned.
#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct Sphere {
    /// The lenth of the line
    pub radius: JitteredValue,

    /// How the spawned particle will be oriented
    pub particle_orientation: SphereParticleOrientation,
}

/// Defines how particles will be oriented when spawned within a sphere shape
#[derive(Debug, Clone, Reflect, FromReflect)]
pub enum SphereParticleOrientation {
    /// Particles will be oriented away from the center of the sphere
    AwayFromCenter,
    /// Particles will be randomly oriented depending on the provided factor
    ///
    /// The Factor will be clamped between 0 and 1
    /// ZERO will have the same behavior as `AwayFromCenter`, ONE will set a completely random orientation
    Random(f32),
    /// Particles will be oriented towards the given direction
    Vector(Vec3),
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            radius: 0.0.into(),
            particle_orientation: SphereParticleOrientation::AwayFromCenter,
        }
    }
}

/// Defines a line along which particles will be spawned.
#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct Cone {
    /// The direction of the cone
    ///
    /// Should be normalized
    pub direction: Vec3,

    /// The angle of the cone (spread) in radians
    ///
    /// Zero indicates straight to the direction. [`std::f32::consts::PI`] indicates a 180 degrees angle (half-sphere)
    pub angle: JitteredValue,

    /// Radius within which the particle can be spawn along the cone
    ///
    /// Zero indicates that the particle will spawn at the particle system position
    pub radius: JitteredValue,
}

impl Default for Cone {
    fn default() -> Self {
        Self {
            direction: Vec3::Z,
            radius: 0.0.into(),
            angle: (0.0..0.5).into(),
        }
    }
}

/// Describes the shape on which new particles get spawned
///
/// For convenience, these can also be created directly from
/// [`CircleSegment`] and [`Line`] instances, or using [`EmitterShape::line`] or
/// [`EmitterShape::circle`]
///
/// # Examples
///
/// ```rust
/// # use bevy_particle_systems::values::{CircleSegment, EmitterShape, Line};
/// # use bevy_particle_systems::ParticleSystem;
/// let particle_system = ParticleSystem {
///     emitter_shape: CircleSegment::default().into(),
///     // ...
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Reflect, FromReflect)]
pub enum EmitterShape {
    /// An oriented 2D segment of a circle with a given radius
    CircleSegment(CircleSegment),
    /// Emit particles from a 2D line at an angle
    Line(Line),
    /// Emit particles within a 3D sphere given its radius
    Sphere(Sphere),
    /// Emit particles from a 3D cone
    Cone(Cone),
}

impl EmitterShape {
    /// Defines a circular emitter with the specified radius.
    ///
    /// See [`CircleSegment`] for more details.
    pub fn circle<T>(radius: T) -> Self
    where
        T: Into<JitteredValue>,
    {
        Self::CircleSegment(CircleSegment {
            radius: radius.into(),
            ..Default::default()
        })
    }

    /// Creates a new Line emitter with the specified length and angle in radian.
    ///
    /// See [`Line`] for more details.
    pub fn line<T>(length: f32, angle: T) -> Self
    where
        T: Into<JitteredValue>,
    {
        Self::Line(Line {
            length,
            angle: angle.into(),
        })
    }

    /// Samples a random starting transform from the Emitter shape
    ///
    /// The returned transform describes the position and direction of movement of the newly spawned particle.
    /// (Note: The actual angle of the new particle might get overridden for a [`crate::components::ParticleSystem`] e.g if
    /// `rotate_to_movement_direction` is false.)
    pub fn sample(&self, rng: &mut ThreadRng) -> Transform {
        match self {
            EmitterShape::CircleSegment(CircleSegment {
                opening_angle,
                radius,
                direction_angle,
            }) => {
                let radian: f32 = rng.gen_range(-0.5..0.5) * opening_angle + direction_angle;
                let direction = Vec3::new(radian.cos(), radian.sin(), 0.0);

                let delta = direction * radius.get_value(rng);
                Transform::from_translation(delta).with_rotation(Quat::from_rotation_z(radian))
            }
            EmitterShape::Line(Line { length, angle }) => {
                let angle = angle.get_value(rng);
                let distance: f32 = rng.gen_range(-0.5..0.5) * length;

                let rotation = Quat::from_rotation_z(angle);

                Transform::from_translation(rotation * vec3(0.0, distance, 0.0))
                    .with_rotation(rotation)
            }
            EmitterShape::Sphere(Sphere {
                radius,
                particle_orientation,
            }) => {
                let spawn_direction = Vec3::new(
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                )
                .normalize();

                let r = radius.get_value(rng).abs();
                let spawn_point = if r == 0.0 {
                    Vec3::splat(0.0)
                } else {
                    let dist = rng.gen_range(0.0..=r);
                    spawn_direction * dist
                };

                match particle_orientation {
                    SphereParticleOrientation::AwayFromCenter => Transform::IDENTITY
                        .looking_at(spawn_direction, spawn_direction.cross(Vec3::Z))
                        .with_translation(spawn_point),
                    SphereParticleOrientation::Random(f) => {
                        let factor = f.clamp(0.0, 1.0);
                        let mut tf = Transform::IDENTITY
                            .looking_at(spawn_direction, spawn_direction.cross(Vec3::Z))
                            .with_translation(spawn_point);

                        if factor == 0.0 {
                            tf
                        } else {
                            let rotation_factor = std::f32::consts::PI * 2.0 * factor;
                            let random_rotation = Quat::from_euler(
                                bevy_math::EulerRot::XYZ,
                                rng.gen_range(-1.0..1.0) * rotation_factor,
                                rng.gen_range(-1.0..1.0) * rotation_factor,
                                rng.gen_range(-1.0..1.0) * rotation_factor,
                            );
                            tf.rotate(random_rotation);
                            tf
                        }
                    }
                    SphereParticleOrientation::Vector(direction) => Transform::IDENTITY
                        .looking_at(*direction, direction.cross(Vec3::Z))
                        .with_translation(spawn_point),
                }
            }
            EmitterShape::Cone(Cone {
                direction,
                angle,
                radius,
            }) => {
                let random_angle = rng.gen_range(0.0..1.0) * 2.0 * std::f32::consts::PI;
                let random_rotation = Quat::from_axis_angle(*direction, random_angle);
                let cross_axis = if *direction == Vec3::Z {
                    Vec3::Y
                } else {
                    Vec3::Z
                };
                let angle_axis = random_rotation * direction.cross(cross_axis);
                let angle = angle.get_value(rng);
                let direction_with_angle = Quat::from_axis_angle(angle_axis, angle) * *direction;

                let tf = Transform::IDENTITY
                    .looking_at(direction_with_angle, direction_with_angle.cross(Vec3::Z));

                let radius = radius.get_value(rng);
                if radius == 0.0 {
                    tf
                } else {
                    let position = *direction * (rng.gen_range(0.0..=1.0) * radius);
                    tf.with_translation(position)
                }
            }
        }
    }
}

impl Default for EmitterShape {
    fn default() -> Self {
        Self::CircleSegment(CircleSegment::default())
    }
}

/// A value that will be chosen from a set of possible values when read.
///
/// ## Examples
///
/// ``T`` values can be converted into ``Constant``
/// [`Range<T>`]s or [`Vec<T>`]s can be converted into ``RandomChoice``
///
/// ## Examples
/// ```
/// # use bevy_particle_systems::values::{RandomValue};
/// # use rand;
///
/// let mut rng = rand::thread_rng();
///
/// // Results in a constant value
/// let c: RandomValue<usize> = (2).into();
///
/// // Results are picked randomly from a range
/// let r: RandomValue<usize> = (1..3).into();
///
/// // Results are picked randomly from a set of values
/// let v: RandomValue<usize> = vec![0, 2, 4, 8].into();
/// ```
#[derive(Debug, Clone, Reflect, FromReflect)]
pub enum RandomValue<T: Reflect + Clone + FromReflect> {
    /// A constant value
    Constant(T),

    /// A set of possible values to choose from randomly
    RandomChoice(Vec<T>),
}

impl<T: Reflect + Clone + FromReflect> From<T> for RandomValue<T> {
    fn from(t: T) -> Self {
        RandomValue::Constant(t)
    }
}

impl<T: Reflect + Clone + FromReflect> From<Range<T>> for RandomValue<T>
where
    Range<T>: Iterator<Item = T>,
{
    fn from(r: Range<T>) -> Self {
        RandomValue::RandomChoice(r.collect())
    }
}

impl<T: Reflect + Clone + FromReflect> From<Vec<T>> for RandomValue<T> {
    fn from(v: Vec<T>) -> Self {
        RandomValue::RandomChoice(v)
    }
}

impl<T: Reflect + Clone + FromReflect> RandomValue<T> {
    /// Get a value from the set of possible values
    ///
    /// # Panics
    ///
    /// Will panic if there are no values to choose from
    pub fn get_value(&self, rng: &mut ThreadRng) -> T {
        match self {
            Self::Constant(t) => t.clone(),
            Self::RandomChoice(v) => {
                assert!(
                    !v.is_empty(),
                    "RandomValue::RandomChoice has no values to choose from!"
                );
                v.choose(rng).unwrap().clone()
            }
        }
    }
}

/// Defines an index of a texture atlas to use for a particle
#[derive(Debug, Clone, Reflect, FromReflect)]
pub enum AtlasIndex {
    /// Constant index
    Constant(usize),
    /// Index randomly choosen at the particle spawn
    Random(RandomValue<usize>),
    /// Animated index, to animate a sprite sheet
    Animated(AnimatedIndex),
}

impl AtlasIndex {
    /// Returns what should be the initial value of the index, at the particle spawn
    pub fn get_value(&self, rng: &mut ThreadRng) -> usize {
        match self {
            Self::Constant(c) => *c,
            Self::Random(r) => r.get_value(rng),
            Self::Animated(a) => a.get_at_start(),
        }
    }
}

impl From<usize> for AtlasIndex {
    fn from(u: usize) -> Self {
        AtlasIndex::Constant(u)
    }
}

impl From<Range<usize>> for AtlasIndex {
    fn from(r: Range<usize>) -> Self {
        AtlasIndex::Random(r.into())
    }
}

impl From<Vec<usize>> for AtlasIndex {
    fn from(v: Vec<usize>) -> Self {
        AtlasIndex::Random(v.into())
    }
}

impl From<f32> for AtlasIndex {
    fn from(t: f32) -> Self {
        AtlasIndex::Animated(AnimatedIndex {
            indices: vec![],
            time_step: t,
            step_offset: 0,
        })
    }
}

impl From<(Range<usize>, f32)> for AtlasIndex {
    fn from((range, time): (Range<usize>, f32)) -> Self {
        AtlasIndex::Animated(AnimatedIndex {
            indices: range.collect(),
            time_step: time,
            step_offset: 0,
        })
    }
}

impl From<(Range<usize>, f32, usize)> for AtlasIndex {
    fn from((range, time, step): (Range<usize>, f32, usize)) -> Self {
        AtlasIndex::Animated(AnimatedIndex {
            indices: range.collect(),
            time_step: time,
            step_offset: step,
        })
    }
}

impl From<(Vec<usize>, f32)> for AtlasIndex {
    fn from((indices, time): (Vec<usize>, f32)) -> Self {
        AtlasIndex::Animated(AnimatedIndex {
            indices,
            time_step: time,
            step_offset: 0,
        })
    }
}

impl From<(Vec<usize>, f32, usize)> for AtlasIndex {
    fn from((indices, time, step): (Vec<usize>, f32, usize)) -> Self {
        AtlasIndex::Animated(AnimatedIndex {
            indices,
            time_step: time,
            step_offset: step,
        })
    }
}

impl From<&TextureAtlas> for AtlasIndex {
    fn from(atlas: &TextureAtlas) -> Self {
        AtlasIndex::Animated(AnimatedIndex {
            indices: { (0..(atlas.len())).collect() },
            time_step: 1.0 / 6.0, // 1/6 seconds is fine
            step_offset: 0,
        })
    }
}

impl From<(&TextureAtlas, f32)> for AtlasIndex {
    fn from((atlas, time): (&TextureAtlas, f32)) -> Self {
        AtlasIndex::Animated(AnimatedIndex {
            indices: { (0..atlas.len()).collect() },
            time_step: time,
            step_offset: 0,
        })
    }
}

impl Default for AtlasIndex {
    fn default() -> Self {
        AtlasIndex::Constant(0)
    }
}

/// A value that has random jitter within a configured range added to it when read.
///
/// Ranges can include negative values as well to return values below the specified base value.
///
/// Generated jitter will be distributed uniformly across the range over time.
///
/// ## Examples
///
/// The following code returns a constant value.
/// ```
/// # use bevy_particle_systems::values::JitteredValue;
/// let mut rng = rand::thread_rng();
/// let jittered_value: JitteredValue = 5.0.into();
/// for _ in 0..10 {
///     // The rng will not be invoked for constant values.
///     let value = jittered_value.get_value(&mut rng);
///     assert_eq!(value, 5.0);
/// }
/// ```
///
/// The following example would have ``get_value`` return values betten `5.0` and `15.0`.
///
/// ```
/// # use bevy_particle_systems::values::JitteredValue;
/// let mut rng = rand::thread_rng();
/// let jittered_value = JitteredValue::jittered(10.0, -5.0..5.0);
/// for _ in 0..10 {
///     let value = jittered_value.get_value(&mut rng);
///     assert!(value < 15.0);
///     assert!(value >= 5.0);
/// }
/// ```
#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct JitteredValue {
    /// The base value that specified jitter will be added to.
    pub value: f32,

    /// A [`Range`] of possible random jitter to be added to ``value`` at evaluation time.
    ///
    /// ``jitter_range`` start value can be negative to allow some values to be less than the base as well.
    pub jitter_range: Option<Range<f32>>,
}

impl JitteredValue {
    /// Create a new value with no jitter.
    pub const fn new(f: f32) -> Self {
        Self {
            value: f,
            jitter_range: None,
        }
    }

    /// Create a new value with a jitter range.
    pub const fn jittered(f: f32, jitter_range: Range<f32>) -> Self {
        Self {
            value: f,
            jitter_range: Some(jitter_range),
        }
    }

    /// Create a new ``JitteredValue`` with a value centered within the jitter range.
    pub fn centered_range(range: Range<f32>) -> Self {
        let mid = (range.start + range.end) / 2.;
        let half_width = (range.end - range.start) / 2.;
        let start = mid - half_width;
        let end = mid + half_width;
        Self {
            value: mid,
            jitter_range: Some(start..end),
        }
    }

    /// Create a new ``JitteredValue`` from an existing one with the specified jitter range.
    pub const fn with_jitter(&self, jitter_range: Range<f32>) -> Self {
        Self {
            value: self.value,
            jitter_range: Some(jitter_range),
        }
    }

    /// Get a value with random jitter within ``jitter_range`` added to it.
    pub fn get_value(&self, rng: &mut ThreadRng) -> f32 {
        match &self.jitter_range {
            Some(r) => self.value + rng.gen_range(r.clone()),
            None => self.value,
        }
    }
}

impl From<f32> for JitteredValue {
    fn from(f: f32) -> Self {
        JitteredValue::new(f)
    }
}

impl From<Range<f32>> for JitteredValue {
    fn from(range: Range<f32>) -> Self {
        JitteredValue::centered_range(range)
    }
}

/// Linearly interpolates between two values by a given percentage.
///
/// ``pct`` should be between `0.0` and `1.0`, but it is up to the trait implementor to ensure
/// that the value is clamped.
///
/// ## Examples
///
/// ```
/// # use bevy_particle_systems::values::Lerpable;
/// # use bevy::prelude::Color;
/// assert_eq!(0.0_f32.lerp(1.0, 0.5), 0.5);
/// assert_eq!(Color::WHITE.lerp(Color::BLACK, 0.5), Color::rgba(0.5, 0.5, 0.5, 1.0));
/// ```
pub trait Lerpable<T> {
    /// Linearly interpolate between the current value and the ``other`` value by ``pct`` percent.
    fn lerp(&self, other: T, pct: f32) -> T;
}

impl Lerpable<f32> for f32 {
    #[inline]
    fn lerp(&self, other: f32, pct: f32) -> f32 {
        lerp(*self, other, pct.clamp(0.0, 1.0))
    }
}

impl Lerpable<Vec3> for Vec3 {
    #[inline]
    fn lerp(&self, other: Vec3, pct: f32) -> Vec3 {
        Vec3::lerp(*self, other, pct.clamp(0.0, 1.0))
    }
}

impl Lerpable<Color> for Color {
    #[inline]
    fn lerp(&self, other: Color, pct: f32) -> Color {
        let clamped_pct = pct.clamp(0.0, 1.0);

        // Convert both colors to float arrays first. Calling `r()`, `g()`, `b()` and `a()`
        // copies the entire struct every time, whereas this should only copy once each.
        // This whas showing up in the hot path when profiling the `basic` example when
        // calling each individually, due to the excessive copies.
        let rgba = self.as_rgba_f32();
        let other_rgba = other.as_rgba_f32();

        Color::rgba(
            rgba[0].lerp(other_rgba[0], clamped_pct),
            rgba[1].lerp(other_rgba[1], clamped_pct),
            rgba[2].lerp(other_rgba[2], clamped_pct),
            rgba[3].lerp(other_rgba[3], clamped_pct),
        )
    }
}

/// Lerp between two floats by ``pct``.
///
/// ``pct`` must be between `0.0` and `1.0` inclusive.
#[inline]
fn lerp(a: f32, b: f32, pct: f32) -> f32 {
    a * (1.0 - pct) + b * pct
}

/// Define the default value returned by a [`Curve`] if misconfigured.
pub trait ErrorDefault<T> {
    /// Define the default value returned by a [`Curve`] if misconfigured.
    fn get_error_default() -> T;
}

impl ErrorDefault<f32> for f32 {
    fn get_error_default() -> f32 {
        0.0
    }
}

impl ErrorDefault<Vec3> for Vec3 {
    fn get_error_default() -> Vec3 {
        Vec3::splat(0.0)
    }
}

impl ErrorDefault<Color> for Color {
    fn get_error_default() -> Color {
        Color::FUCHSIA
    }
}

/// Determines whether or not two values of an imprecise type are close enough to call equal.
///
/// Provides implementations for ``f32`` and ``f64`` using [`std::f32::EPSILON`] and [`std::f64::EPSILON`] as the max allowable difference.
///
/// ## Examples
/// ```
/// # use bevy_particle_systems::values::RoughlyEqual;
/// assert!(0.0_f32.roughly_equal(0.0000001));
/// assert!(!0.0_f32.roughly_equal(0.000001));
/// assert!(0.0_f64.roughly_equal(0.00000000000000001));
/// ```
pub trait RoughlyEqual<T> {
    /// Evalues whether the current value is roughly equal to ``other`` within the types maximum allowable difference.
    fn roughly_equal(&self, other: T) -> bool;
}

impl RoughlyEqual<f32> for f32 {
    #[inline]
    fn roughly_equal(&self, other: f32) -> bool {
        (self - other).abs() < f32::EPSILON
    }
}

impl RoughlyEqual<f64> for f64 {
    #[inline]
    fn roughly_equal(&self, other: f64) -> bool {
        (self - other).abs() < f64::EPSILON
    }
}

/// Defines a value at a specific point in a curve.
///
/// ``point`` should be between `0.0` and `1.0` inclusive.
#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct CurvePoint<T>
where
    T: Lerpable<T> + ErrorDefault<T> + Copy + Reflect + FromReflect,
{
    /// Defines the value at a specified point in time.
    pub value: T,
    /// Defines the point in time at which exactly ``value`` will be the presented value.
    ///
    /// The returned value of an evaluation of the curve will be lerped between the two closest [`CurvePoint`]s based on their ``point`` value.
    pub point: f32,
}

impl<T> CurvePoint<T>
where
    T: Lerpable<T> + ErrorDefault<T> + Copy + Reflect + FromReflect,
{
    /// Create a new [`CurvePoint`] of the specified ``value`` at the given ``point``.
    ///
    /// ``point`` should be between `0.0` and `1.0` inclusive.
    pub fn new(value: T, point: f32) -> Self {
        Self { value, point }
    }
}

/// Defines a curve as a series of curve points.
///
/// A [`Curve`] should always contain at least two [`CurvePoint`]s,
/// one at `0.0` and one at `1.0`.
///
/// Computing the curve without state is a linear operation and can add up to be
/// somewhat expensive. [`Curve::sample_mut`] can be used in these scenarios to potentialy
/// improve performance, as long as the particular curve only moves forward in time. This
/// will use an `index_hint` state to skip to where the previous call was in curve detection.
///
/// If most or all of the curves are only two components, it is likely better to use [`Curve::sample`]
/// rather than [`Curve::sample_mut`], as both will take the same shortcuts, but [`Curve::sample`] does not
/// require a mutable borrow and therefore can be used in parallel with other systems.
///
/// ## Examples
/// ```
/// # use bevy::prelude::Color;
/// # use bevy_particle_systems::values::{CurvePoint, Curve};
/// let curve = Curve::new(vec![CurvePoint::new(Color::BLACK, 0.0), CurvePoint::new(Color::WHITE, 1.0)]);
/// assert_eq!(curve.sample(0.5), Color::rgba(0.5, 0.5, 0.5, 1.0));
///
/// let three_color_curve = Curve::new(vec![CurvePoint::new(Color::BLACK, 0.0), CurvePoint::new(Color::WHITE, 0.5), CurvePoint::new(Color::BLACK, 1.0)]);
/// assert_eq!(three_color_curve.sample(0.5), Color::rgba(1.0, 1.0, 1.0, 1.0));
/// assert_eq!(three_color_curve.sample(0.75), Color::rgba(0.5, 0.5, 0.5, 1.0));
///
/// let alpha_curve = Curve::new(vec![CurvePoint::new(Color::rgba(1.0, 1.0, 1.0, 1.0), 0.0), CurvePoint::new(Color::rgba(1.0, 1.0, 1.0, 0.0), 1.0)]);
/// assert_eq!(alpha_curve.sample(0.5), Color::rgba(1.0, 1.0, 1.0, 0.5));
/// ```
#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct Curve<T>
where
    T: Lerpable<T> + ErrorDefault<T> + Copy + Reflect + FromReflect,
{
    points: Vec<CurvePoint<T>>,
    index_hint: usize,
}

impl<T> Curve<T>
where
    T: Lerpable<T> + ErrorDefault<T> + Copy + Reflect + FromReflect,
{
    /// Creates a new Curve from given [`CurvePoint`]s.
    ///
    /// Points should be in sorted, ascending order. There must be at least two points.
    /// The first point must be at 0.0 and the last at 1.0.
    ///
    /// This function panics in dev builds if this is not the case.
    pub fn new(points: Vec<CurvePoint<T>>) -> Self {
        debug_assert!(
            points.len() >= 2,
            "Cannot have a gradient with less than two colors"
        );

        debug_assert!(
            points[0].point.roughly_equal(0.0),
            "Gradient must start at 0.0"
        );

        debug_assert!(
            points[points.len() - 1].point.roughly_equal(1.0),
            "Gradients must end at 1.0"
        );

        #[cfg(dev)]
        for i in 1..points.len() {
            debug_assert!(
                points[i - 1].point < points[i].point,
                "Gradient points must be sorted, with no identical points"
            );
        }

        Self {
            points,
            index_hint: 0,
        }
    }

    /// Get the value at ``pct`` percentage of the way through the curve.
    ///
    /// ``pct`` will be clamped between 0.0 and 1.0.
    ///
    /// Returns [`ErrorDefault::get_error_default`] as a fallback if no value is found for ``pct``. This indicates
    /// that the curve is misconfigured.
    ///
    /// Sets the internal `index_hint` to the index of the value found so subsequent calls of a `pct` greater than the
    /// current call will be faster. This is only useful for curvess which have more than two [`CurvePoint`]s, otherwise,
    /// use [`Curve::sample`] instead. If `pct` is less than a previous call for this curve, `index_hint` will be reset. The
    /// resulting color for these call should always be correct, but may result in a performance hit if done out of order.
    #[inline]
    pub fn sample_mut(&mut self, pct: f32) -> T {
        let clamped_pct = pct.clamp(0.0, 1.0);

        // Shortcuts
        if clamped_pct == 0.0 {
            return self.points[0].value;
        }

        if clamped_pct.roughly_equal(1.0) {
            return self.points[self.points.len() - 1].value;
        }

        // If there's only two values just directly lerp between them.
        if self.points.len() == 2 {
            return self.points[0].value.lerp(
                self.points[1].value,
                (clamped_pct - self.points[0].point)
                    / (self.points[1].point - self.points[0].point).abs(),
            );
        }

        // If pct is not moving forward, reset the index hint to zero so we can just scan from the beginning again.
        if clamped_pct < self.points[self.index_hint].point {
            self.index_hint = 0;
        }

        let mut current_point = self.points[self.index_hint].point;
        let mut current_value = self.points[self.index_hint].value;
        let mut next_point = self.points[self.index_hint + 1].point;
        let mut next_value = self.points[self.index_hint + 1].value;

        if self.index_hint <= self.points.len() - 2
            && clamped_pct >= current_point
            && clamped_pct < next_point
        {
            return current_value.lerp(
                next_value,
                (clamped_pct - current_point) / (next_point - current_point).abs(),
            );
        }

        // Find the first value where the point is less than `pct`, starting from the last index that was used,
        // indicating we need to lerp between that value and the next value. This requires points in the vec to
        // be sorted to behave correctly.
        for i in self.index_hint..self.points.len() - 1 {
            current_point = self.points[i].point;
            current_value = self.points[i].value;
            next_point = self.points[i + 1].point;
            next_value = self.points[i + 1].value;

            if current_point.roughly_equal(clamped_pct) {
                return current_value;
            }

            if clamped_pct > current_point && clamped_pct < next_point {
                self.index_hint = i;
                return current_value.lerp(
                    next_value,
                    (clamped_pct - current_point) / (next_point - current_point).abs(),
                );
            }
            continue;
        }

        T::get_error_default()
    }

    /// Get the value at ``pct`` percentage of the way through the curve.
    ///
    /// ``pct`` will be clamped between 0.0 and 1.0.
    ///
    /// Returns [`ErrorDefault::get_error_default`] as a fallback if no value is found for ``pct``. This indicates
    /// that the curve is misconfigured.
    ///
    /// This operation is linear with the number of [`CurvePoint`]s contained in the curve. If curvess contain more than
    /// two [`CurvePoint`]s, it may be faster to use `sample_mut`, which does index tracking.
    pub fn sample(&self, pct: f32) -> T {
        let clamped_pct = pct.clamp(0.0, 1.0);

        // Shortcuts
        if clamped_pct == 0.0 {
            return self.points[0].value;
        }

        if clamped_pct.roughly_equal(1.0) {
            return self.points[self.points.len() - 1].value;
        }

        // If there's only two colors just directly lerp between them.
        if self.points.len() == 2 {
            return self.points[0].value.lerp(
                self.points[1].value,
                (clamped_pct - self.points[0].point)
                    / (self.points[1].point - self.points[0].point).abs(),
            );
        }

        // Find the first value where the point is less than `pct`, indicating we need to
        // lerp between that value and the next value. This requires points in the vec to
        // be sorted to behave correctly.
        for i in 0..self.points.len() - 1 {
            if self.points[i].point.roughly_equal(clamped_pct) {
                return self.points[i].value;
            }

            if clamped_pct > self.points[i].point && clamped_pct < self.points[i + 1].point {
                return self.points[i].value.lerp(
                    self.points[i + 1].value,
                    (clamped_pct - self.points[i].point)
                        / (self.points[i + 1].point - self.points[i].point).abs(),
                );
            }
            continue;
        }

        T::get_error_default()
    }
}

/// Defines how a color changes over time
///
/// Colors can either be constant, linearly interpolated, or follow a [`crate::values::Curve`].
#[derive(Debug, Clone, Reflect, FromReflect)]
pub enum ColorOverTime {
    /// Specifies that a color should remain a constant color over time.
    Constant(Color),

    /// Specifies that a color should be linearly interpolated between two colors over time.
    Lerp(Lerp<Color>),

    /// Specifies that a color will follow a curve of two or more colors over time.
    Gradient(Curve<Color>),
}

impl Default for ColorOverTime {
    fn default() -> Self {
        ColorOverTime::Constant(Color::WHITE)
    }
}

impl From<Color> for ColorOverTime {
    fn from(color: Color) -> Self {
        ColorOverTime::Constant(color)
    }
}

impl From<Range<Color>> for ColorOverTime {
    fn from(r: Range<Color>) -> Self {
        ColorOverTime::Lerp(Lerp::new(r.start, r.end))
    }
}

impl From<Vec<CurvePoint<Color>>> for ColorOverTime {
    fn from(gradient: Vec<CurvePoint<Color>>) -> Self {
        if gradient.len() == 2 && gradient[0].point <= 0.0 && gradient[1].point >= 1.0 {
            ColorOverTime::Lerp(Lerp::new(gradient[0].value, gradient[1].value))
        } else {
            ColorOverTime::Gradient(Curve::new(gradient))
        }
    }
}

impl ColorOverTime {
    /// Evaluate a color at the specified lifetime percentage.
    ///
    /// ``pct`` should be between `0.0` and `1.0` inclusive.
    pub fn at_lifetime_pct(&self, pct: f32) -> Color {
        match self {
            Self::Constant(c) => *c,
            Self::Lerp(l) => l.a.lerp(l.b, pct),
            Self::Gradient(g) => g.sample(pct),
        }
    }
}

/// Defines how a vector changes over time
///
/// Vectors can either be constant, linearly interpolated, or follow a [`crate::values::Curve`].
#[derive(Debug, Clone, Reflect, FromReflect)]
pub enum VectorOverTime {
    /// Specifies that a color should remain a constant color over time.
    Constant(Vec3),

    /// Specifies that a color should be linearly interpolated between two color over time.
    Lerp(Lerp<Vec3>),

    /// Specifies that a color will follow a curve of two or more colors over time.
    Gradient(Curve<Vec3>),
}

impl Default for VectorOverTime {
    fn default() -> Self {
        VectorOverTime::Constant(Vec3::splat(0.0))
    }
}

impl From<Vec3> for VectorOverTime {
    fn from(vector: Vec3) -> Self {
        VectorOverTime::Constant(vector)
    }
}

impl From<Range<Vec3>> for VectorOverTime {
    fn from(r: Range<Vec3>) -> Self {
        VectorOverTime::Lerp(Lerp::new(r.start, r.end))
    }
}

impl From<Vec<CurvePoint<Vec3>>> for VectorOverTime {
    fn from(curve: Vec<CurvePoint<Vec3>>) -> Self {
        if curve.len() == 2 && curve[0].point <= 0.0 && curve[1].point >= 1.0 {
            VectorOverTime::Lerp(Lerp::new(curve[0].value, curve[1].value))
        } else {
            VectorOverTime::Gradient(Curve::new(curve))
        }
    }
}

impl VectorOverTime {
    /// Evaluate a color at the specified lifetime percentage.
    ///
    /// ``pct`` should be between `0.0` and `1.0` inclusive.
    pub fn at_lifetime_pct(&self, pct: f32) -> Vec3 {
        match self {
            Self::Constant(v) => *v,
            Self::Lerp(l) => l.a.lerp(l.b, pct),
            Self::Gradient(g) => g.sample(pct),
        }
    }
}

/// Defines several methods for modifying a value over time.
///
/// ``f32`` values can be converted into ``Constant`` and [`Range<f32>`]s can be converted into
/// [`Lerp`] values.
///
/// ## Examples
/// ```
/// # use bevy_particle_systems::values::{Lerp, RoughlyEqual, SinWave, ValueOverTime};
/// // Results in a Lerp value
/// let l: ValueOverTime = (0.0_f32..1.0).into();
/// assert_eq!(l.at_lifetime_pct(0.5), 0.5);
///
/// // Results in a constant value.
/// let c: ValueOverTime = 1.0.into();
/// assert_eq!(c.at_lifetime_pct(0.5), 1.0);
///
/// let s = ValueOverTime::Sin(SinWave::new());
/// assert!(s.at_lifetime_pct(0.0).roughly_equal(0.0));
/// assert!(s.at_lifetime_pct(0.25).roughly_equal(1.0));
/// assert!(s.at_lifetime_pct(0.5).roughly_equal(0.0));
/// assert!(s.at_lifetime_pct(0.75).roughly_equal(-1.0));
/// ```
#[derive(Debug, Clone, Reflect, FromReflect)]
pub enum ValueOverTime {
    /// Specifies the value should be linearly interpolated between two values over time.
    Lerp(Lerp<f32>),

    /// Specifies that a color will follow a gradient of two or more colors over time.
    Curve(Curve<f32>),

    /// Specifies that the value should follow a sinusoidal wave over time.
    ///
    /// The value will complete [`SinWave::period`] full waves over its lifetime.
    Sin(SinWave),

    /// Specifies that the value should remain constant.
    Constant(f32),
}

impl From<f32> for ValueOverTime {
    fn from(f: f32) -> Self {
        ValueOverTime::Constant(f)
    }
}

impl From<Range<f32>> for ValueOverTime {
    fn from(r: Range<f32>) -> Self {
        ValueOverTime::Lerp(Lerp::new(r.start, r.end))
    }
}

impl From<Vec<CurvePoint<f32>>> for ValueOverTime {
    fn from(curve: Vec<CurvePoint<f32>>) -> Self {
        ValueOverTime::Curve(Curve::new(curve))
    }
}

impl ValueOverTime {
    /// Gets the value at the specified percentage of its lifetime
    pub fn at_lifetime_pct(&self, pct: f32) -> f32 {
        match self {
            Self::Lerp(l) => l.a.lerp(l.b, pct),
            Self::Curve(c) => c.sample(pct),
            Self::Sin(s) => {
                s.amplitude * (s.period * (pct * std::f32::consts::TAU) - s.phase_shift).sin()
                    + s.vertical_shift
            }
            Self::Constant(c) => *c,
        }
    }
}

/// Defines a value that will linearly move between ``a`` and ``b`` over its configured lifetime.
#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct Lerp<T: Lerpable<T>> {
    /// The starting value, returned when ``pct`` is `0.0`.
    pub a: T,
    /// The ending value, returned when ``pct`` is `1.0`.
    pub b: T,
}

impl<T: Lerpable<T>> Lerp<T> {
    /// Create a new [`Lerp`] to move between ``a`` and ``b`` values over time.
    pub const fn new(a: T, b: T) -> Self {
        Self { a, b }
    }
}

impl Default for Lerp<f32> {
    fn default() -> Self {
        Self { a: 0.0, b: 1.0 }
    }
}

impl Default for Lerp<Vec3> {
    fn default() -> Self {
        Self {
            a: Vec3::splat(0.0),
            b: Vec3::splat(1.0),
        }
    }
}

impl Default for Lerp<Color> {
    fn default() -> Self {
        Self {
            a: Color::BLACK,
            b: Color::WHITE,
        }
    }
}

/// Defines a value that will move in a sinusoidal wave pattern over it's configured lifetime.
#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct SinWave {
    /// The amplitude of the wave as time progresses.
    ///
    /// This determines how far above and below the baseline (default of `0.0`, modified with ``vertical_shift``) the wave will go.
    pub amplitude: f32,
    /// The number of times a full wave will complete over the lifetime.
    ///
    /// If the both the ``amplitude`` and ``period`` are `1.0`, the wave will hit both `1.0` and `-1.0` once over its lifetime return to `0.0` at the end.
    pub period: f32,
    /// How far left or right to shift the starting point of the wave.
    pub phase_shift: f32,
    /// How far vertically to shift the wave.
    ///
    /// If a wave should not have a negative value, this must be at least ``amplitude``, which causes the maximum value to be `2 * amplitude`.
    pub vertical_shift: f32,
}

impl SinWave {
    /// Create a new default wave with one full wave of 0 -> 1 -> 0 -> -1 -> 0
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for SinWave {
    fn default() -> Self {
        Self {
            amplitude: 1.0,
            period: 1.0,
            phase_shift: 0.0,
            vertical_shift: 0.0,
        }
    }
}

#[derive(Debug, Clone, Reflect, FromReflect)]
/// Defines a flow field that will influence particles velocity over space and time.
pub struct Noise2D {
    /// Frequency of the noise.
    ///
    /// Increase for wiggling effect, decrease for smooth waves.
    pub frequency: f32,
    /// Amplitude of the noise.
    ///
    /// Defines how much the noise will affect the particles.
    pub amplitude: f32,
    /// How time affects the noise.
    pub time_factor: f32,
    /// Translation of the noise.
    ///
    /// Defines how much the noise will change over time in X and Y axis.
    pub translation: Vec2,
}
impl Default for Noise2D {
    fn default() -> Self {
        Self {
            frequency: 0.1,
            amplitude: 100.0,
            time_factor: 1.0,
            translation: Vec2::new(10.0, 8.5),
        }
    }
}
impl Noise2D {
    /// Creates a new `Noise2D`
    pub fn new(frequency: f32, amplitude: f32, time_factor: f32, translation: Vec2) -> Self {
        Noise2D {
            frequency,
            amplitude,
            time_factor,
            translation,
        }
    }

    /// Evaluates the noise at a given position and time
    pub fn sample(&self, position: Vec2, time: f32) -> Vec2 {
        let n1 = 128.648; // random number useful to compute noise
        let n2 = 0.8614;
        let sampling_position = position + self.translation * time * self.time_factor;
        let sample_x = (sampling_position.x * self.frequency).sin_cos();
        let sample_y = ((sampling_position.y + n1) * (self.frequency * n2)).sin_cos();

        Vec2::new(sample_x.0 + sample_y.1, sample_x.1 + sample_y.0) * self.amplitude
    }
}

#[derive(Debug, Clone, Reflect, FromReflect)]
/// Defines a flow field that will influence particles velocity over space and time.
pub struct Noise3D {
    /// Frequency of the noise.
    ///
    /// Increase for wiggling effect, decrease for smooth waves.
    pub frequency: f32,
    /// Amplitude of the noise.
    ///
    /// Defines how much the noise will affect the particles.
    pub amplitude: f32,
    /// How time affects the noise.
    pub time_factor: f32,
    /// Translation of the noise.
    ///
    /// Defines how much the noise will change over time in X and Y axis.
    pub translation: Vec3,
}
impl Default for Noise3D {
    fn default() -> Self {
        Self {
            frequency: 1.0,
            amplitude: 5.0,
            time_factor: 1.0,
            translation: Vec3::new(10.0, 8.5, 5.3),
        }
    }
}
impl Noise3D {
    /// Creates a new `Noise2D`
    pub fn new(frequency: f32, amplitude: f32, time_factor: f32, translation: Vec3) -> Self {
        Noise3D {
            frequency,
            amplitude,
            time_factor,
            translation,
        }
    }

    /// Evaluates the noise at a given position and time
    pub fn sample(&self, position: Vec3, time: f32) -> Vec3 {
        let n1_y = 128.648; // random numbers useful to compute noise
        let n2_y = 0.8614;
        let n1_z = 53.168;
        let n2_z = 1.1359;
        let sampling_position = position + self.translation * time * self.time_factor;
        let sample_x = (sampling_position.x * self.frequency).sin_cos();
        let sample_y = ((sampling_position.y + n1_y) * (self.frequency * n2_y)).sin_cos();
        let sample_z = ((sampling_position.z + n1_z) * (self.frequency * n2_z)).sin_cos();

        Vec3::new(
            sample_z.0 + sample_y.1,
            sample_x.1 + sample_z.0,
            sample_x.0 + sample_y.0,
        ) * self.amplitude
    }
}

/// Defines an acceleration modifier that will affect particles velocity.
#[derive(Debug, Clone, Reflect, FromReflect)]
pub enum VelocityModifier {
    /// f32 value that will use the direction of the current velocity.
    Scalar(ValueOverTime),
    /// Constant vector acceleration, such as gravity.
    Vector(VectorOverTime),
    /// Force that will slow down the particles like air resistance.
    Drag(ValueOverTime),
    /// Sinusoidal 2D Noise
    Noise2D(Noise2D),
    /// Sinusoidal 3D Noise
    Noise3D(Noise3D),
}

impl From<Noise2D> for VelocityModifier {
    fn from(value: Noise2D) -> Self {
        Self::Noise2D(value)
    }
}

impl From<Noise3D> for VelocityModifier {
    fn from(value: Noise3D) -> Self {
        Self::Noise3D(value)
    }
}

/// Setup optional values used so that every calculated values are not re-calculated for every modifiers that uses it
/// Use it to get particle square speed, speed and direction
pub struct PrecalculatedParticleVariables {
    /// velocity squared length
    pub particle_sqr_speed: Option<f32>,
    /// velocity length
    pub particle_speed: Option<f32>,
    /// velocity normalized
    pub particle_direction: Option<Vec3>,
}

impl PrecalculatedParticleVariables {
    /// Creates a new empty `PrecalculatedParticleValues`
    pub fn new() -> Self {
        PrecalculatedParticleVariables {
            particle_sqr_speed: None,
            particle_speed: None,
            particle_direction: None,
        }
    }
    /// Return or Calculate particle squared speed (velocity squared length)
    pub fn get_particle_sqr_speed(&mut self, velocity: &Vec3) -> f32 {
        if let Some(x) = self.particle_sqr_speed {
            return x;
        }

        let result = velocity.length_squared();
        self.particle_sqr_speed = Some(result);
        result
    }
    /// Return or Calculate particle speed (velocity length)
    pub fn get_particle_speed(&mut self, velocity: &Vec3) -> f32 {
        if let Some(x) = self.particle_speed {
            return x;
        }

        let result = self.get_particle_sqr_speed(velocity).sqrt();
        self.particle_speed = Some(result);
        result
    }
    /// Return or Calculate particle direction (velocity normalized)
    pub fn get_particle_direction(&mut self, velocity: &Vec3) -> Vec3 {
        if let Some(x) = self.particle_direction {
            return x;
        }

        let result = *velocity / self.get_particle_speed(velocity);
        self.particle_direction = Some(result);
        result
    }
}

impl Default for PrecalculatedParticleVariables {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Reflect, FromReflect)]
/// This type describes how particles should be aligned with their velocity
pub enum VelocityAlignedType {
    /// X will put the particle local X axis along the velocity vector
    X,
    /// NegativeX will put the particle local -X axis along the velocity vector
    NegativeX,
    /// Y will put the particle local Y axis along the velocity vector
    Y,
    /// NegativeY will put the particle local -Y axis along the velocity vector
    NegativeY,
    /// Z will put the particle local Z axis along the velocity vector
    Z,
    /// NegativeZ will put the particle local -Z axis along the velocity vector
    NegativeZ,
    /// Custom will align with the provided local space vector. It should be normalized for it to work as intended
    CustomLocal(Vec3),
    /// Custom will align with the provided vector. It should be normalized for it to work as intended
    CustomGlobal(Vec3),
}

impl VelocityAlignedType {
    /// Provide a vector to calculate the particle velocity alignment for [`crate::ParticleRenderType::Billboard3d`]
    ///  # Panics
    ///
    /// Will panic if [`VelocityAlignedType::CustomGlobal`] alignment is used
    pub fn get_billboard_alignment(&self) -> Vec3 {
        match self {
            VelocityAlignedType::X => Vec3::X,
            VelocityAlignedType::NegativeX => -Vec3::X,
            VelocityAlignedType::Y => Vec3::Y,
            VelocityAlignedType::NegativeY => -Vec3::Y,
            VelocityAlignedType::Z | VelocityAlignedType::NegativeZ => Vec3::ZERO,
            VelocityAlignedType::CustomLocal(v) => Vec3::new(v.x, v.y, 0.0).normalize(),
            VelocityAlignedType::CustomGlobal(_) => {
                panic!("CustomGlobal alignment not supported for Billboard rendering");
            }
        }
    }
}
