use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::{App, Camera2dBundle, ClearColor, Color, Commands, Res},
    window::{PresentMode, WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};
use bevy_app::PluginGroup;
use bevy_asset::AssetServer;
use bevy_particle_systems::{
    JitteredValue, ParticleSystem, ParticleSystemBundle, ParticleSystemPlugin, ParticleTexture,
    Playing,
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
                texture: ParticleTexture::Sprite(asset_server.load("arrow.png")),
                spawn_rate_per_second: 25.0.into(),
                spawn_radius: 10.0.into(),
                initial_speed: JitteredValue::jittered(70.0, -3.0..3.0),
                lifetime: JitteredValue::jittered(5.0, -1.0..1.0),
                emitter_shape: std::f32::consts::PI,
                emitter_angle: std::f32::consts::PI / 2.0,
                looping: true,
                scale: 0.07.into(),
                system_duration_seconds: 5.0,
                initial_rotation: (-90.0_f32).to_radians().into(),
                rotate_to_movement_direction: true,
                ..ParticleSystem::default()
            },
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);
}
