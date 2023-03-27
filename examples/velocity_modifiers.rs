//! This example demonstrates the how velocity modifiers works:
//! The red particle system only has a constant acceleration that makes particles accelerate upwards over time.
//! The blue one only has a drag effect that makes particles slow down over time.
//! The orange one combine both constant acceleration and drag.
//! 
//! There is no limit but performance in how much velocity modifiers a particle system can have simultaneously.

use bevy::{
    math::Vec3,
    prelude::{App, Camera2dBundle, Color, Commands, Component, Res, Transform},
    DefaultPlugins,
};
use bevy_asset::AssetServer;

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
        .run();
}

fn startup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 500,
                emitter_shape: CircleSegment {
                    opening_angle: std::f32::consts::PI * 0.15,
                    ..Default::default()
                }
                .into(),
                texture: ParticleTexture::Sprite(asset_server.load("px.png")),
                spawn_rate_per_second: 40.0.into(),
                initial_speed: JitteredValue::jittered(300.0, -100.0..100.0),
                velocity_modifiers: vec![
                    // This will make the particles go up
                    ConstantVector(Vec3::new(0.0, 250.0, 0.0)),
                ],
                lifetime: JitteredValue::jittered(1.5, -0.2..0.2),
                color: ColorOverTime::Constant(Color::GREEN),
                looping: true,
                system_duration_seconds: 10.0,
                space: ParticleSpace::World,
                scale: 10.0.into(),
                rotation_speed: 2.0.into(),
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(-350.0, 100.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);

    commands
        .spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 500,
                emitter_shape: CircleSegment {
                    opening_angle: std::f32::consts::PI * 0.15,
                    ..Default::default()
                }
                .into(),
                texture: ParticleTexture::Sprite(asset_server.load("px.png")),
                spawn_rate_per_second: 40.0.into(),
                initial_speed: JitteredValue::jittered(300.0, -100.0..100.0),
                velocity_modifiers: vec![
                    // This will make them slow down
                    Drag(0.01.into()),
                ],
                lifetime: JitteredValue::jittered(1.5, -0.2..0.2),
                color: ColorOverTime::Constant(Color::RED),
                system_duration_seconds: 10.0,
                scale: 10.0.into(),
                rotation_speed: 2.0.into(),
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(-350.0, -100.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);

    
    commands
        .spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 500,
                emitter_shape: CircleSegment {
                    opening_angle: std::f32::consts::PI * 0.15,
                    ..Default::default()
                }
                .into(),
                texture: ParticleTexture::Sprite(asset_server.load("px.png")),
                spawn_rate_per_second: 40.0.into(),
                initial_speed: JitteredValue::jittered(300.0, -100.0..100.0),
                velocity_modifiers: vec![
                    // This will make the particles go up
                    ConstantVector(Vec3::new(0.0, 250.0, 0.0)),
                    // This will make them slow down
                    Drag(0.01.into()),
                    // the variant VelocityModifier::Value() can be seen the example basic.rs
                ],
                lifetime: JitteredValue::jittered(1.5, -0.2..0.2),
                color: ColorOverTime::Gradient(Gradient::new(vec![
                    ColorPoint::new(Color::WHITE, 0.0),
                    ColorPoint::new(Color::rgba(0.8, 0.2, 0.0, 1.0), 0.05),
                    ColorPoint::new(Color::rgba(0.0, 0.0, 0.0, 0.25), 0.5),
                    ColorPoint::new(Color::rgba(0.0, 0.0, 0.0, 0.0), 1.0),
                ])),
                scale: (8.0..50.0).into(),
                rotation_speed: 2.0.into(),
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(200.0, 0.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);
}