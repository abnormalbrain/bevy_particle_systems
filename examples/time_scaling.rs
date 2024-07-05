//! This example demonstrates how time scaling impacts particle systems.
//!
//! The red particles do not follow scaled time, while the green do.
//! Time scale can be controls with the 1-5 keys. The 0 key sets the Time Scale to 0.0
//! which effectively pauses the system.
use bevy::{
    input::ButtonInput,
    prelude::{App, Camera2dBundle, Color, Commands, KeyCode, Res, ResMut, Transform},
    DefaultPlugins,
};
use bevy_app::{Startup, Update};
use bevy_asset::AssetServer;
use bevy_color::palettes::basic::*;
use bevy_particle_systems::{
    CircleSegment, ColorOverTime, Curve, CurvePoint, JitteredValue, ParticleSpace, ParticleSystem,
    ParticleSystemBundle, ParticleSystemPlugin, Playing,
};
use bevy_time::{Time, Virtual};
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ParticleSystemPlugin)) // <-- Add the plugin
        .add_systems(Startup, startup_system)
        .add_systems(Update, time_scale_changer)
        .run();
}

fn startup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 500,
                emitter_shape: CircleSegment {
                    direction_angle: 0.0,
                    opening_angle: std::f32::consts::PI * 0.25,
                    radius: 0.0.into(),
                }
                .into(),
                texture: asset_server.load("px.png").into(),
                spawn_rate_per_second: 35.0.into(),
                initial_speed: JitteredValue::jittered(25.0, 0.0..5.0),
                lifetime: JitteredValue::jittered(3.0, -2.0..2.0),
                color: ColorOverTime::Gradient(Curve::new(vec![
                    CurvePoint::new(RED.into(), 0.0),
                    CurvePoint::new(Color::rgba(0.0, 0.0, 0.0, 0.0), 1.0),
                ])),
                looping: true,
                system_duration_seconds: 10.0,
                space: ParticleSpace::World,
                scale: 5.0.into(),
                initial_rotation: JitteredValue::jittered(0.0, -2.0..2.0),
                use_scaled_time: false,
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(50.0, 0.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);

    commands
        .spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 500,
                emitter_shape: CircleSegment {
                    opening_angle: std::f32::consts::PI * 0.25,
                    direction_angle: std::f32::consts::PI,
                    radius: 0.0.into(),
                }
                .into(),
                texture: asset_server.load("px.png").into(),
                spawn_rate_per_second: 35.0.into(),
                initial_speed: JitteredValue::jittered(25.0, 0.0..5.0),
                lifetime: JitteredValue::jittered(3.0, -2.0..2.0),
                color: ColorOverTime::Gradient(Curve::new(vec![
                    CurvePoint::new(GREEN.into(), 0.0),
                    CurvePoint::new(Color::rgba(0.0, 0.0, 0.0, 0.0), 1.0),
                ])),
                looping: true,
                system_duration_seconds: 10.0,
                space: ParticleSpace::World,
                scale: 5.0.into(),
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(-50.0, 0.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);
}

fn time_scale_changer(keys: Res<ButtonInput<KeyCode>>, mut time: ResMut<Time<Virtual>>) {
    for key_code in keys.get_just_pressed() {
        match key_code {
            KeyCode::Digit1 => time.set_relative_speed(1.0),
            KeyCode::Digit2 => time.set_relative_speed(2.0),
            KeyCode::Digit3 => time.set_relative_speed(4.0),
            KeyCode::Digit4 => time.set_relative_speed(8.0),
            KeyCode::Digit5 => time.set_relative_speed(10.0),
            KeyCode::Digit0 => time.set_relative_speed(0.0),
            _ => {}
        }
    }
}
