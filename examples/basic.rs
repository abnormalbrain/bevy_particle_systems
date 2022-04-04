use bevy::{
    prelude::{App, AssetServer, Color, Commands, OrthographicCameraBundle, Res},
    DefaultPlugins,
};
use bevy_particles::{
    components::{ParticleBurst, ParticleSystem, ParticleSystemBundle, Playing},
    plugin::ParticleSystemPlugin,
    values::{ColorOverTime, ColorPoint, Gradient, JitteredValue, SinWave, ValueOverTime},
};

fn main() {
    App::new()
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
                max_particles: 50_000,
                default_sprite: asset_server.load("px.png"),
                spawn_rate_per_second: 1000.0.into(),
                initial_velocity: JitteredValue::jittered(3.0, -1.0..1.0),
                acceleration: ValueOverTime::Sin(SinWave::new()),
                lifetime: JitteredValue::jittered(8.0, -2.0..2.0),
                color: ColorOverTime::Gradient(Gradient::new(vec![
                    ColorPoint::new(Color::PURPLE, 0.0),
                    ColorPoint::new(Color::RED, 0.5),
                    ColorPoint::new(Color::rgba(0.0, 0.0, 1.0, 0.0), 1.0),
                ])),
                looping: true,
                system_duration_seconds: 10.0,
                scale: 2.0.into(),
                bursts: vec![
                    ParticleBurst::new(0.0, 5000),
                    ParticleBurst::new(2.0, 5000),
                    ParticleBurst::new(4.0, 5000),
                    ParticleBurst::new(6.0, 5000),
                    ParticleBurst::new(8.0, 5000),
                ],
                ..ParticleSystem::default()
            },
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);
}
