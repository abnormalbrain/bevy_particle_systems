//! A shader that renders a mesh multiple times in one draw call.

use std::f32::consts::PI;
use bevy::prelude::*;
use bevy_particle_systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ParticleSystemPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .add_system(rotate_camera)
        .run();
}

#[derive(Resource)]
pub struct IsCheck;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Blue Particles
    commands.spawn(ParticleSystemBundle {
        particle_system: ParticleSystem {
            // Must be set for 3D rendering
            // Note that this is equivalent as the default settings: ParticleRenderType::Billboard3d(default())
            render_type: ParticleRenderType::Billboard3d(Billboard3dSettings {
                use_frustrum_culling: false,
                // Here, particles are almost always opaque, so we skip this expensive sorting
                sort_particles_by_depth: false,
            }),
            emitter_shape: EmitterShape::Sphere(Sphere::default()),
            max_particles: 50_000,
            spawn_rate_per_second: 1000.0.into(),
            initial_speed: JitteredValue::jittered(2.0, -1.0..1.0),
            initial_rotation: (-PI..PI).into(),
            rotation_speed: (-10.0..10.0).into(),
            velocity_modifiers: vec![
                VelocityModifier::Drag(0.01.into()),
                Noise3D {
                    amplitude: 1.0,
                    time_factor: 0.5,
                    ..Default::default()
                }.into(),
                ],
            lifetime: 3.5.into(),
            color: ColorOverTime::Gradient(Curve::new(vec![
                CurvePoint::new(Color::WHITE, 0.0),
                CurvePoint::new(Color::rgba(1.0, 1.0, 0.5, 1.0), 0.1),
                CurvePoint::new(Color::BLUE, 0.7),
                CurvePoint::new(Color::rgba(0.0, 0.0, 0.5, 1.0), 0.9),
                CurvePoint::new(Color::rgba(0.0, 0.0, 0.0, 0.0), 1.0),
            ])),
            looping: true,
            system_duration_seconds: 10.0,
            scale: (0.1..0.05).into(),
            ..ParticleSystem::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..ParticleSystemBundle::default()
    })
    .insert(Playing);


    // Arrows
    commands.spawn(ParticleSystemBundle {
        particle_system: ParticleSystem {
            render_type: ParticleRenderType::Billboard3d(Billboard3dSettings {
                use_frustrum_culling: false,
                // Here, the arrow particles are overlapping each other a lot with transparency,
                // we then need to sort them before rendering or we will encounter bad alpha blending
                sort_particles_by_depth: true,
            }),
            emitter_shape: EmitterShape::Cone(Cone {
                direction: Vec3::Z,
                angle: (0.00..0.01).into(),
                ..Default::default()
            }),
            texture: ParticleTexture::Sprite(asset_server.load("arrow.png")),
            max_particles: 1000,
            spawn_rate_per_second: 5.0.into(),
            initial_speed: 2.0.into(),
            align_with_velocity: Some(VelocityAlignedType::NegativeY),
            velocity_modifiers: vec![VelocityModifier::Drag(0.01.into())],
            lifetime: 3.5.into(),
            color: ColorOverTime::Gradient(Curve::new(vec![
                CurvePoint::new(Color::WHITE, 0.0),
                CurvePoint::new(Color::WHITE, 0.5),
                CurvePoint::new(Color::rgba(1.0, 1.0, 1.0, 0.0), 1.0),
            ])),
            looping: true,
            system_duration_seconds: 10.0,
            scale: 0.5.into(),
            ..ParticleSystem::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..ParticleSystemBundle::default()
    })
    .insert(Playing);

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn rotate_camera (
    mut cam_query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    let mut tf = cam_query.get_single_mut().unwrap();
    let rot = Quat::from_rotation_z(0.5 * time.delta_seconds());
    tf.rotate_around(Vec3::ZERO, rot);
    tf.look_at(Vec3::ZERO, Vec3::Y);
}