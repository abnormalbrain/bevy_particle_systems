//! This example demonstrates the how the drag slows down particles.

use bevy::{
    math::Vec3,
    prelude::{App, Camera2dBundle, Color, Commands, Component, Query, Res, Transform, With},
    DefaultPlugins,
};
use bevy_asset::AssetServer;
use bevy_math::Quat;
use bevy_time::Time;

use bevy_particle_systems::{
    CircleSegment, ColorOverTime, ColorPoint, Gradient, JitteredValue, ParticleSpace,
    ParticleSystem, ParticleSystemBundle, ParticleSystemPlugin, ParticleTexture, Playing, VelocityModifier::*,
};

#[derive(Debug, Component)]
pub struct Targets {
    pub targets: Vec<Vec3>,
    pub index: usize,
    pub time: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ParticleSystemPlugin::default()) // <-- Add the plugin
        .add_startup_system(startup_system)
        .add_system(circler)
        .run();
}

fn startup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 500,
                emitter_shape: CircleSegment {
                    opening_angle: std::f32::consts::PI * 0.085,
                    ..Default::default()
                }
                .into(),
                texture: ParticleTexture::Sprite(asset_server.load("px.png")),
                spawn_rate_per_second: 40.0.into(),
                initial_speed: JitteredValue::jittered(600.0, -100.0..100.0),
                velocity_modifiers: vec![
                    // This will make the particles go up
                    ConstantVector(Vec3::new(0.0, 500.0, 0.0)),
                    // This will make them slow down
                    Drag(0.02.into()),
                    // For VelocityModifier::Value(), see the example basic.rs
                ],
                lifetime: JitteredValue::jittered(1.5, -0.2..0.2),
                color: ColorOverTime::Gradient(Gradient::new(vec![
                    ColorPoint::new(Color::WHITE, 0.0),
                    ColorPoint::new(Color::rgba(0.8, 0.2, 0.0, 1.0), 0.05),
                    ColorPoint::new(Color::rgba(0.0, 0.0, 0.0, 0.25), 0.5),
                    ColorPoint::new(Color::rgba(0.0, 0.0, 0.0, 0.0), 1.0),
                ])),
                looping: true,
                system_duration_seconds: 10.0,
                space: ParticleSpace::World,
                scale: (8.0..50.0).into(),
                rotation_speed: 2.0.into(),
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(50.0, 50.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing)
        .insert(Circler::new(Vec3::new(50.0, 0.0, 0.0), 100.0));
}

#[derive(Component)]
pub struct Circler {
    pub center: Vec3,
    pub radius: f32,
}

impl Circler {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}

pub fn circler(
    time: Res<Time>,
    mut particle_system_query: Query<(&Circler, &mut Transform), With<ParticleSystem>>,
) {
    let rad = time.elapsed_seconds() * 2.0;
    let quat = Quat::from_rotation_z(rad).normalize();
    let dir = quat * Vec3::Y;
    for (circler, mut transform) in &mut particle_system_query {
        transform.translation = circler.center + dir * circler.radius;
        transform.rotate_axis(Vec3::Z, 6.0 * time.delta_seconds());
    }
}
