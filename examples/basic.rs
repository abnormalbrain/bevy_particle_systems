use bevy::{
    core_pipeline::ClearColor,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::{App, AssetServer, Color, Commands, OrthographicCameraBundle, Res},
    window::{PresentMode, WindowDescriptor},
    DefaultPlugins,
};
use bevy_particle_systems::{
    ColorOverTime, ColorPoint, Gradient, JitteredValue, ParticleBurst, ParticleSystem,
    ParticleSystemBundle, ParticleSystemPlugin, Playing, SinWave, ValueOverTime,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            present_mode: PresentMode::Immediate,
            ..WindowDescriptor::default()
        })
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(ParticleSystemPlugin::default()) // <-- Add the plugin
        .add_startup_system(startup_system)
        .run();
}

fn startup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands
        .spawn_bundle(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 10_000,
                default_sprite: asset_server.load("px.png"),
                spawn_rate_per_second: 500.0.into(),
                initial_velocity: JitteredValue::jittered(3.0, -1.0..1.0),
                acceleration: ValueOverTime::Sin(SinWave {
                    amplitude: 150.0,
                    period: 5.0,
                    ..SinWave::default()
                }),
                lifetime: JitteredValue::jittered(8.0, -2.0..2.0),
                color: ColorOverTime::Gradient(Gradient::new(vec![
                    ColorPoint::new(Color::PURPLE, 0.0),
                    ColorPoint::new(Color::RED, 0.5),
                    ColorPoint::new(Color::rgba(0.0, 0.0, 1.0, 0.0), 1.0),
                ])),
                looping: true,
                system_duration_seconds: 10.0,
                max_distance: Some(300.0),
                scale: 2.0.into(),
                bursts: vec![
                    ParticleBurst::new(0.0, 1000),
                    ParticleBurst::new(2.0, 1000),
                    ParticleBurst::new(4.0, 1000),
                    ParticleBurst::new(6.0, 1000),
                    ParticleBurst::new(8.0, 1000),
                ],
                ..ParticleSystem::default()
            },
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);
}
