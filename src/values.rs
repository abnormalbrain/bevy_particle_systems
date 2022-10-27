//! Different value types and controls used in particle systems.
use std::ops::Range;

use bevy::render::prelude::Color;
use rand::{prelude::ThreadRng, Rng};

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
#[derive(Debug, Clone)]
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
    fn lerp(&self, other: f32, pct: f32) -> f32 {
        lerp(*self, other, pct.clamp(0.0, 1.0))
    }
}

impl Lerpable<Color> for Color {
    fn lerp(&self, other: Color, pct: f32) -> Color {
        let clamped_pct = pct.clamp(0.0, 1.0);
        Color::rgba(
            self.r().lerp(other.r(), clamped_pct),
            self.g().lerp(other.g(), clamped_pct),
            self.b().lerp(other.b(), clamped_pct),
            self.a().lerp(other.a(), clamped_pct),
        )
    }
}

/// Lerp between two floats by ``pct``.
///
/// ``pct`` must be between `0.0` and `1.0` inclusive.
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone)]
pub struct Gradient(Vec<ColorPoint>);

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

        Self(points)
    }

    /// Get the color at ``pct`` percentage of the way through the gradient.
    ///
    /// ``pct`` will be clamped between 0.0 and 1.0.
    ///
    /// Returns [`bevy::prelude::Color::FUCHSIA`] as a fallback if no color is found for ``pct``. This indicates
    /// that the gradient is misconfigured.
    pub fn get_color(&self, pct: f32) -> Color {
        let clamped_pct = pct.clamp(0.0, 1.0);

        // Shortcuts
        if clamped_pct == 0.0 {
            return self.0[0].color;
        }

        if clamped_pct.roughly_equal(1.0) {
            return self.0[self.0.len() - 1].color;
        }

        // If there's only two colors just directly lerp between them.
        if self.0.len() == 2 {
            return self.0[0].color.lerp(
                self.0[1].color,
                (clamped_pct - self.0[0].point) / (self.0[1].point - self.0[0].point).abs(),
            );
        }

        // Find the first color where the point is less than `pct`, indicating we need to
        // lerp between that color and the next color. This requires points in the vec to
        // be sorted to behave correctly.
        for i in 0..self.0.len() - 1 {
            if self.0[i].point.roughly_equal(clamped_pct) {
                return self.0[i].color;
            }

            if clamped_pct > self.0[i].point && clamped_pct < self.0[i + 1].point {
                return self.0[i].color.lerp(
                    self.0[i + 1].color,
                    (clamped_pct - self.0[i].point) / (self.0[i + 1].point - self.0[i].point).abs(),
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
