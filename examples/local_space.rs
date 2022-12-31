//! This example demonstrates the difference between using particles in local and global space.
//!
//! The red colored particles operate in global space. Once they have been spawned they move independently.
//! The green particles operate in local space. You can see that their movement is affected by the movement of the spawn point as well.
use bevy::{
    math::Vec3,
    prelude::{App, Camera2dBundle, Color, Commands, Component, Query, Res, Transform, With},
    DefaultPlugins,
};
use bevy_asset::AssetServer;
use bevy_math::Quat;
use bevy_time::Time;

use bevy_particle_systems::{
    ColorOverTime, ColorPoint, Gradient, JitteredValue, ParticleSpace, ParticleSystem,
    ParticleSystemBundle, ParticleSystemPlugin, ParticleTexture, Playing,
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
                emitter_shape: std::f32::consts::PI * 0.25,
                emitter_angle: 0.0,
                texture: ParticleTexture::Sprite(asset_server.load("px.png")),
                spawn_rate_per_second: 35.0.into(),
                initial_speed: JitteredValue::jittered(25.0, 0.0..5.0),
                acceleration: 0.0.into(),
                lifetime: JitteredValue::jittered(3.0, -2.0..2.0),
                color: ColorOverTime::Gradient(Gradient::new(vec![
                    ColorPoint::new(Color::RED, 0.0),
                    ColorPoint::new(Color::rgba(0.0, 0.0, 0.0, 0.0), 1.0),
                ])),
                looping: true,
                system_duration_seconds: 10.0,
                space: ParticleSpace::World,
                scale: 8.0.into(),
                rotation_speed: 2.0.into(),
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(50.0, 50.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing)
        .insert(Circler::new(Vec3::new(50.0, 0.0, 0.0), 50.0));

    commands
        .spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 500,
                emitter_shape: std::f32::consts::PI * 0.25,
                emitter_angle: std::f32::consts::PI,
                texture: ParticleTexture::Sprite(asset_server.load("px.png")),
                spawn_rate_per_second: 35.0.into(),
                initial_speed: JitteredValue::jittered(25.0, 0.0..5.0),
                acceleration: 0.0.into(),
                lifetime: JitteredValue::jittered(3.0, -2.0..2.0),
                color: ColorOverTime::Gradient(Gradient::new(vec![
                    ColorPoint::new(Color::GREEN, 0.0),
                    ColorPoint::new(Color::rgba(0.0, 0.0, 0.0, 0.0), 1.0),
                ])),
                looping: true,
                system_duration_seconds: 10.0,
                space: ParticleSpace::Local,
                scale: 8.0.into(),
                rotation_speed: JitteredValue::jittered(0.0, -6.0..0.0),
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(-50.0, 50.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing)
        .insert(Circler::new(Vec3::new(-50.0, 0.0, 0.0), 50.0));
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
    let rad = time.elapsed_seconds() as f32;
    let quat = Quat::from_rotation_z(rad).normalize();
    let dir = quat * Vec3::Y;
    for (circler, mut transform) in &mut particle_system_query {
        transform.translation = circler.center + dir * circler.radius;
    }
}
