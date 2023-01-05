//! This example demonstrates how time scaling impacts particle systems.
//!
//! The red particles do not follow scaled time, while the green do.
//! Time scale can be controls with the 1-5 keys. The 0 key sets the Time Scale to 0.0
//! which effectively pauses the system.
use bevy::{
    input::Input,
    prelude::{App, Camera2dBundle, Color, Commands, KeyCode, Res, ResMut, Transform},
    DefaultPlugins,
};
use bevy_asset::AssetServer;
use bevy_particle_systems::{
    ColorOverTime, ColorPoint, Gradient, JitteredValue, ParticleSpace, ParticleSystem,
    ParticleSystemBundle, ParticleSystemPlugin, ParticleTexture, Playing,
};
use bevy_time::Time;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ParticleSystemPlugin::default()) // <-- Add the plugin
        .add_startup_system(startup_system)
        .add_system(time_scale_changer)
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
                space: ParticleSpace::World,
                scale: 5.0.into(),
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(-50.0, 0.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);
}

fn time_scale_changer(keys: Res<Input<KeyCode>>, mut time: ResMut<Time>) {
    for key_code in keys.get_just_pressed() {
        match key_code {
            KeyCode::Key1 => time.set_relative_speed(1.0),
            KeyCode::Key2 => time.set_relative_speed(2.0),
            KeyCode::Key3 => time.set_relative_speed(4.0),
            KeyCode::Key4 => time.set_relative_speed(8.0),
            KeyCode::Key5 => time.set_relative_speed(10.0),
            KeyCode::Key0 => time.set_relative_speed(0.0),
            _ => {}
        }
    }
}
