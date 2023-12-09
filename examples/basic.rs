use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{PresentMode, WindowPlugin},
    DefaultPlugins,
};
use bevy_particle_systems::{
    ColorOverTime, Curve, CurvePoint, JitteredValue, ParticleBurst, ParticleSystem,
    ParticleSystemBundle, ParticleSystemPlugin, Playing, VelocityModifier::*,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            EntityCountDiagnosticsPlugin,
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ParticleSystemPlugin::default()) // <-- Add the plugin
        .add_systems(Startup, startup_system)
        .run();
}

fn startup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 50_000,
                texture: asset_server.load("px.png").into(),
                spawn_rate_per_second: 1000.0.into(),
                initial_speed: JitteredValue::jittered(200.0, -50.0..50.0),
                velocity_modifiers: vec![Drag(0.01.into())],
                lifetime: JitteredValue::jittered(8.0, -2.0..2.0),
                color: ColorOverTime::Gradient(Curve::new(vec![
                    CurvePoint::new(Color::PURPLE, 0.0),
                    CurvePoint::new(Color::RED, 0.5),
                    CurvePoint::new(Color::rgba(0.0, 0.0, 1.0, 0.0), 1.0),
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
