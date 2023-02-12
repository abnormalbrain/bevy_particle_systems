use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::{App, Camera2dBundle, ClearColor, Color, Commands, Res, Transform},
    window::{PresentMode, WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};
use bevy_app::PluginGroup;
use bevy_asset::AssetServer;

use bevy_particle_systems::{
    CircleSegment, ColorOverTime, ColorPoint, EmitterShape, Gradient, JitteredValue,
    ParticleSystem, ParticleSystemBundle, ParticleSystemPlugin, ParticleTexture, Playing,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugin(EntityCountDiagnosticsPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                present_mode: PresentMode::AutoNoVsync,
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_plugin(ParticleSystemPlugin::default()) // <-- Add the plugin
        .add_startup_system(startup_system)
        .run();
}

fn startup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 50_000,
                texture: ParticleTexture::Sprite(asset_server.load("arrow.png")),
                spawn_rate_per_second: 10.0.into(),
                initial_speed: JitteredValue::jittered(70.0, -3.0..3.0),
                lifetime: JitteredValue::jittered(10.0, -2.0..2.0),
                color: ColorOverTime::Gradient(Gradient::new(vec![
                    ColorPoint::new(Color::PURPLE, 0.0),
                    ColorPoint::new(Color::RED, 0.5),
                    ColorPoint::new(Color::rgba(0.0, 0.0, 1.0, 0.0), 1.0),
                ])),
                emitter_shape: EmitterShape::line(200.0, std::f32::consts::FRAC_PI_4),
                looping: true,
                rotate_to_movement_direction: true,
                initial_rotation: (-90.0_f32).to_radians().into(),
                system_duration_seconds: 10.0,
                max_distance: Some(300.0),
                scale: 0.07.into(),
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(0.0, -200.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);

    commands
        .spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 50_000,
                texture: ParticleTexture::Sprite(asset_server.load("arrow.png")),
                spawn_rate_per_second: 10.0.into(),
                initial_speed: JitteredValue::jittered(70.0, -3.0..3.0),
                lifetime: JitteredValue::jittered(10.0, -2.0..2.0),
                color: ColorOverTime::Gradient(Gradient::new(vec![
                    ColorPoint::new(Color::PURPLE, 0.0),
                    ColorPoint::new(Color::RED, 0.5),
                    ColorPoint::new(Color::rgba(0.0, 0.0, 1.0, 0.0), 1.0),
                ])),
                emitter_shape: EmitterShape::CircleSegment(CircleSegment {
                    radius: 10.0.into(),
                    opening_angle: std::f32::consts::PI,
                    direction_angle: std::f32::consts::FRAC_PI_4,
                }),
                looping: true,
                rotate_to_movement_direction: true,
                initial_rotation: (-90.0_f32).to_radians().into(),
                system_duration_seconds: 10.0,
                max_distance: Some(300.0),
                scale: 0.07.into(),
                ..ParticleSystem::default()
            },
            transform: Transform::from_xyz(0.0, 200.0, 0.0),
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);
}
