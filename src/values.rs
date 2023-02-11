//! Different value types and controls used in particle systems.
use std::ops::Range;

use bevy_math::{vec3, Quat, Vec3};
use bevy_reflect::{FromReflect, Reflect};
use bevy_render::prelude::Color;
use bevy_transform::prelude::Transform;
use rand::seq::SliceRandom;
use rand::{prelude::ThreadRng, Rng};

/// Describes the shape on which new particles get spawned
#[derive(Debug, Clone, Reflect, FromReflect)]
pub enum EmitterShape {
    /// A oriented segment of a circle at a given radius
    CircleSegment {
        /// The shape of the emitter, defined in radian.
        ///
        /// The default is [`std::f32::consts::TAU`], which results particles going in all directions in a circle.
        /// Reducing the value reduces the possible emitting directions. [`std::f32::consts::PI`] will emit particles
        /// in a semi-circle.
        opening_angle: f32,

        /// The rotation angle of the emitter, defined in radian.
        ///
        /// Zero indicates straight to the right in the X direction. [`std::f32::consts::PI`] indicates straight left in the X direction.
        direction_angle: f32,

        /// The radius around the particle systems location that particles will spawn in.
        ///
        /// Setting this to zero will make all particles start at the same position.
        /// Setting this to a non-jittered constant will make particles spawn exactly that distance away from the
        /// center position. Jitter will allow particles to spawn in a range.
        radius: JitteredValue,
    },
    /// Emit particles from a 2d line at an angle
    Line {
        /// The lenth of the line
        length: f32,

        /// The rotation angle of the emitter, defined in radian.
        ///
        /// Zero indicates straight to the right in the +X direction. [`std::f32::consts::PI`] indicates straight left in the -X direction.
        angle: JitteredValue,
    },
}

impl EmitterShape {
    /// Samples a random starting transform from the Emitter shape
    ///
    /// The returned transform describes the position and direction of movement of the newly spawned particle.
    /// (Note: The actual angle of the new particle might get overridden for a [`crate::components::ParticleSystem`] e.g if
    /// `rotate_to_movement_direction` is false.)
    pub fn sample(&self, rng: &mut ThreadRng) -> Transform {
        match self {
            EmitterShape::CircleSegment {
                opening_angle,
                radius,
                direction_angle,
            } => {
                let radian: f32 = rng.gen_range(-0.5..0.5) * opening_angle + direction_angle;
                let direction = Vec3::new(radian.cos(), radian.sin(), 0.0);

                let delta = direction * radius.get_value(rng);
                Transform::from_translation(delta).with_rotation(Quat::from_rotation_z(radian))
            }
            EmitterShape::Line { length, angle } => {
                let angle = angle.get_value(rng);
                let distance: f32 = rng.gen_range(-0.5..0.5) * length;

                let rotation = Quat::from_rotation_z(angle);

                Transform::from_translation(rotation * vec3(0.0, distance, 0.0))
                    .with_rotation(rotation)
            }
        }
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

    /// Create a new ``JitteredValue`` with a value centered withing the jitter range.
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

/// Defines a color at a specific point in a gradient.
///
/// ``point`` should be between `0.0` and `1.0` inclusive.
#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct ColorPoint {
    /// Defines the [`Color`] value at a specified point in time.
    pub color: Color,

    /// Defines the point in time at which exactly this [`Color`] will be the presented value.
    ///
    /// The returned color of an evaluation of the gradient will be lerped between the two closest [`ColorPoint`]s based on their ``point`` value.
    pub point: f32,
}

impl ColorPoint {
    /// Create a new [`ColorPoint`] of the specified [`Color`] at the given ``point``.
    ///
    /// ``point`` should be between `0.0` and `1.0` inclusive.
    pub fn new(color: Color, point: f32) -> Self {
        Self { color, point }
    }
}

/// Defines a gradient as a series of color points.
///
/// A [`Gradient`] should always contain at least two [`ColorPoint`]s,
/// one at `0.0` and one at `1.0`.
///
/// Computing the gradient without state is a linear operation and can add up to be
/// somewhat expensive. [`Gradient::get_color_mut`] can be used in these scenarios to potentialy
/// improve performance, as long as the particular gradient only moves forward in time. This
/// will use an `index_hint` state to skip to where the previous call was in gradient detection.
///
/// If most or all of the gradients are only two components, it is likely better to use [`Gradient::get_color`]
/// rather than [`Gradient::get_color_mut`], as both will take the same shortcuts, but [`Gradient::get_color`] does not
/// require a mutable borrow and therefore can be used in parallel with other systems.
///
/// ## Examples
/// ```
/// # use bevy::prelude::Color;
/// # use bevy_particle_systems::values::{ColorPoint, Gradient};
/// let gradient = Gradient::new(vec![ColorPoint::new(Color::BLACK, 0.0), ColorPoint::new(Color::WHITE, 1.0)]);
/// assert_eq!(gradient.get_color(0.5), Color::rgba(0.5, 0.5, 0.5, 1.0));
///
/// let three_color_gradient = Gradient::new(vec![ColorPoint::new(Color::BLACK, 0.0), ColorPoint::new(Color::WHITE, 0.5), ColorPoint::new(Color::BLACK, 1.0)]);
/// assert_eq!(three_color_gradient.get_color(0.5), Color::rgba(1.0, 1.0, 1.0, 1.0));
/// assert_eq!(three_color_gradient.get_color(0.75), Color::rgba(0.5, 0.5, 0.5, 1.0));
///
/// let alpha_gradient = Gradient::new(vec![ColorPoint::new(Color::rgba(1.0, 1.0, 1.0, 1.0), 0.0), ColorPoint::new(Color::rgba(1.0, 1.0, 1.0, 0.0), 1.0)]);
/// assert_eq!(alpha_gradient.get_color(0.5), Color::rgba(1.0, 1.0, 1.0, 0.5));
/// ```
#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct Gradient {
    points: Vec<ColorPoint>,
    index_hint: usize,
}

impl Gradient {
    /// Creates a new Gradient from given [`ColorPoint`]s.
    ///
    /// Points should be in sorted, ascending order. There must be at least two points.
    /// The first point must be at 0.0 and the last at 1.0.
    ///
    /// This function panics in dev builds if this is not the case.
    pub fn new(points: Vec<ColorPoint>) -> Self {
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

    /// Get the color at ``pct`` percentage of the way through the gradient.
    ///
    /// ``pct`` will be clamped between 0.0 and 1.0.
    ///
    /// Returns [`bevy_render::prelude::Color::FUCHSIA`] as a fallback if no color is found for ``pct``. This indicates
    /// that the gradient is misconfigured.
    ///
    /// Sets the internal `index_hint` to the index of the color found so subsequent calls of a `pct` greater than the
    /// current call will be faster. This is only useful for gradients which have more than two [`ColorPoint`]s, otherwise,
    /// use [`Gradient::get_color`] instead. If `pct` is less than a previous call for this gradient, `index_hint` will be reset. The
    /// resulting color for these call should always be correct, but may result in a performance hit if done out of order.
    #[inline]
    pub fn get_color_mut(&mut self, pct: f32) -> Color {
        let clamped_pct = pct.clamp(0.0, 1.0);

        // Shortcuts
        if clamped_pct == 0.0 {
            return self.points[0].color;
        }

        if clamped_pct.roughly_equal(1.0) {
            return self.points[self.points.len() - 1].color;
        }

        // If there's only two colors just directly lerp between them.
        if self.points.len() == 2 {
            return self.points[0].color.lerp(
                self.points[1].color,
                (clamped_pct - self.points[0].point)
                    / (self.points[1].point - self.points[0].point).abs(),
            );
        }

        // If pct is not moving forward, reset the index hint to zero so we can just scan from the beginning again.
        if clamped_pct < self.points[self.index_hint].point {
            self.index_hint = 0;
        }

        let mut current_point = self.points[self.index_hint].point;
        let mut current_color = self.points[self.index_hint].color;
        let mut next_point = self.points[self.index_hint + 1].point;
        let mut next_color = self.points[self.index_hint + 1].color;

        if self.index_hint <= self.points.len() - 2
            && clamped_pct >= current_point
            && clamped_pct < next_point
        {
            return current_color.lerp(
                next_color,
                (clamped_pct - current_point) / (next_point - current_point).abs(),
            );
        }

        // Find the first color where the point is less than `pct`, starting from the last index that was used,
        // indicating we need to lerp between that color and the next color. This requires points in the vec to
        // be sorted to behave correctly.
        for i in self.index_hint..self.points.len() - 1 {
            current_point = self.points[i].point;
            current_color = self.points[i].color;
            next_point = self.points[i + 1].point;
            next_color = self.points[i + 1].color;

            if current_point.roughly_equal(clamped_pct) {
                return current_color;
            }

            if clamped_pct > current_point && clamped_pct < next_point {
                self.index_hint = i;
                return current_color.lerp(
                    next_color,
                    (clamped_pct - current_point) / (next_point - current_point).abs(),
                );
            }
            continue;
        }

        Color::FUCHSIA
    }

    /// Get the color at ``pct`` percentage of the way through the gradient.
    ///
    /// ``pct`` will be clamped between 0.0 and 1.0.
    ///
    /// Returns [`bevy_render::prelude::Color::FUCHSIA`] as a fallback if no color is found for ``pct``. This indicates
    /// that the gradient is misconfigured.
    ///
    /// This operation is linear with the number of [`ColorPoint`]s contained in the gradient. If gradients contain more than
    /// two [`ColorPoint`]s, it may be faster to use `get_color_mut`, which does index tracking.
    pub fn get_color(&self, pct: f32) -> Color {
        let clamped_pct = pct.clamp(0.0, 1.0);

        // Shortcuts
        if clamped_pct == 0.0 {
            return self.points[0].color;
        }

        if clamped_pct.roughly_equal(1.0) {
            return self.points[self.points.len() - 1].color;
        }

        // If there's only two colors just directly lerp between them.
        if self.points.len() == 2 {
            return self.points[0].color.lerp(
                self.points[1].color,
                (clamped_pct - self.points[0].point)
                    / (self.points[1].point - self.points[0].point).abs(),
            );
        }

        // Find the first color where the point is less than `pct`, indicating we need to
        // lerp between that color and the next color. This requires points in the vec to
        // be sorted to behave correctly.
        for i in 0..self.points.len() - 1 {
            if self.points[i].point.roughly_equal(clamped_pct) {
                return self.points[i].color;
            }

            if clamped_pct > self.points[i].point && clamped_pct < self.points[i + 1].point {
                return self.points[i].color.lerp(
                    self.points[i + 1].color,
                    (clamped_pct - self.points[i].point)
                        / (self.points[i + 1].point - self.points[i].point).abs(),
                );
            }
            continue;
        }

        Color::FUCHSIA
    }
}

/// Defines how a color changes over time
///
/// Colors can either be constant, or follow a [`crate::values::Gradient`].
#[derive(Debug, Clone, Reflect)]
pub enum ColorOverTime {
    /// Specifies that a color should remain a constant color over time.
    Constant(Color),

    /// Specifies that a color will follow a gradient of two or more colors over time.
    Gradient(Gradient),
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

impl From<Vec<ColorPoint>> for ColorOverTime {
    fn from(gradient: Vec<ColorPoint>) -> Self {
        ColorOverTime::Gradient(Gradient::new(gradient))
    }
}

impl ColorOverTime {
    /// Evaluate a color at the specified lifetime percentage.
    ///
    /// ``pct`` should be between `0.0` and `1.0` inclusive.
    pub fn at_lifetime_pct(&self, pct: f32) -> Color {
        match self {
            Self::Constant(color) => *color,
            Self::Gradient(gradient) => gradient.get_color(pct),
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
#[derive(Debug, Clone, Reflect)]
pub enum ValueOverTime {
    /// Specifies the value should be linearly interpolated between two values over time.
    Lerp(Lerp),

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

impl ValueOverTime {
    /// Gets the value at the specified percentage of its lifetime
    pub fn at_lifetime_pct(&self, pct: f32) -> f32 {
        match self {
            Self::Lerp(l) => l.a.lerp(l.b, pct),
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
pub struct Lerp {
    /// The starting value, returned when ``pct`` is `0.0`.
    pub a: f32,
    /// The ending value, returned when ``pct`` is `1.0`.
    pub b: f32,
}

impl Lerp {
    /// Create a new [`Lerp`] to move between ``a`` and ``b`` values over time.
    pub const fn new(a: f32, b: f32) -> Self {
        Self { a, b }
    }
}

impl Default for Lerp {
    fn default() -> Self {
        Self { a: 0.0, b: 1.0 }
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
