//! This example shows how particle systems can be spawned dynamically and
//! automatically despawned when finished.

use bevy::{input::common_conditions::input_just_pressed, prelude::*, window::PrimaryWindow};
use bevy_color::palettes::basic::*;
use bevy_particle_systems::{
    ParticleBurst, ParticleSystem, ParticleSystemBundle, ParticleSystemPlugin, Playing,
    VelocityModifier,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, ParticleSystemPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            spawn_particle_systems.run_if(input_just_pressed(MouseButton::Left)),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_particle_systems(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = camera.single();

    if let Some(world_position) = window
        .single()
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        commands.spawn((
            ParticleSystemBundle {
                transform: Transform::from_translation(world_position.extend(0.)),
                particle_system: ParticleSystem {
                    spawn_rate_per_second: 0.0.into(),
                    max_particles: 1_000,
                    initial_speed: (0.0..300.0).into(),
                    scale: 2.0.into(),
                    velocity_modifiers: vec![
                        VelocityModifier::Drag(0.001.into()),
                        VelocityModifier::Vector(Vec3::new(0.0, -400.0, 0.0).into()),
                    ],
                    color: (BLUE.into()..Color::srgba(1.0, 0.0, 0.0, 0.0)).into(),
                    bursts: vec![ParticleBurst {
                        time: 0.0,
                        count: 1000,
                    }],
                    ..ParticleSystem::oneshot()
                },
                ..default()
            },
            Playing,
        ));
    }
}
