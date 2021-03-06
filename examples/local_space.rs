//! This example demonstrates the difference between using particles in local and global space.
//!
//! The red colored particles operate in global space. Once they have been spawned they move independently.
//! The green particles operate in local space. You can see that their movement is affected by the movement of the spawn point as well.
use bevy::{
    core::Time,
    math::Vec3,
    prelude::{
        App, AssetServer, Color, Commands, Component, OrthographicCameraBundle, Query, Res,
        Transform, With,
    },
    DefaultPlugins,
};
use bevy_particle_systems::{
    ColorOverTime, ColorPoint, Gradient, JitteredValue, Lerpable, ParticleSpace, ParticleSystem,
    ParticleSystemBundle, ParticleSystemPlugin, Playing,
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
        .add_system(mover_system)
        .run();
}

fn startup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands
        .spawn_bundle(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 500,
                emitter_shape: std::f32::consts::PI * 0.25,
                emitter_angle: 0.0,
                default_sprite: asset_server.load("px.png"),
                spawn_rate_per_second: 35.0.into(),
                initial_velocity: JitteredValue::jittered(25.0, 0.0..5.0),
                acceleration: 0.0.into(),
                lifetime: JitteredValue::jittered(3.0, -2.0..2.0),
                color: ColorOverTime::Gradient(Gradient::new(vec![
                    ColorPoint::new(Color::RED, 0.0),
                    ColorPoint::new(Color::rgba(0.0, 0.0, 0.0, 0.0), 1.0),
                ])),
                looping: true,
                system_duration_seconds: 10.0,
                space: ParticleSpace::World,
                scale: 5.0.into(),
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(50.0, 50.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing)
        .insert(Targets {
            targets: vec![Vec3::new(50.0, 100.0, 0.0), Vec3::new(50.0, -100.0, 0.0)],
            index: 0,
            time: 0.0,
        });

    commands
        .spawn_bundle(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 500,
                emitter_shape: std::f32::consts::PI * 0.25,
                emitter_angle: std::f32::consts::PI,
                default_sprite: asset_server.load("px.png"),
                spawn_rate_per_second: 35.0.into(),
                initial_velocity: JitteredValue::jittered(25.0, 0.0..5.0),
                acceleration: 0.0.into(),
                lifetime: JitteredValue::jittered(3.0, -2.0..2.0),
                color: ColorOverTime::Gradient(Gradient::new(vec![
                    ColorPoint::new(Color::GREEN, 0.0),
                    ColorPoint::new(Color::rgba(0.0, 0.0, 0.0, 0.0), 1.0),
                ])),
                looping: true,
                system_duration_seconds: 10.0,
                space: ParticleSpace::Local,
                scale: 5.0.into(),
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(-50.0, 50.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing)
        .insert(Targets {
            targets: vec![Vec3::new(-50.0, 100.0, 0.0), Vec3::new(-50.0, -100.0, 0.0)],
            index: 0,
            time: 0.0,
        });
}

pub fn mover_system(
    time: Res<Time>,
    mut particle_system_query: Query<(&mut Targets, &mut Transform), With<ParticleSystem>>,
) {
    let delta = time.delta_seconds();
    for (mut targets, mut transform) in particle_system_query.iter_mut() {
        let to = targets.targets[targets.index];
        let from_index = if targets.index == 0 {
            targets.targets.len() - 1
        } else {
            targets.index - 1
        };
        let from = targets.targets[from_index];
        targets.time = (targets.time + delta).clamp(0.0, 3.0);

        let pct = targets.time / 3.0;
        transform.translation = Vec3::new(from.x.lerp(to.x, pct), from.y.lerp(to.y, pct), 0.0);

        if targets.time == 3.0 {
            targets.index += 1;
            if targets.index >= targets.targets.len() {
                targets.index = 0;
            }
            targets.time = 0.0;
        }
    }
}
