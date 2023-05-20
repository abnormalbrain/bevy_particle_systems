//! A shader that renders a mesh multiple times in one draw call.

use bevy::prelude::*;
use bevy_particle_systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ParticleSystemPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        //.add_system(setup)
        .run();
}

#[derive(Resource)]
pub struct IsCheck;

fn setup(
    mut commands: Commands,
    check: Option<Res<IsCheck>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Some(_) = check {
        return;
    } else {
        commands.insert_resource(IsCheck);
    }

    /*commands.spawn(ParticleSystemBundle {
        particle_system: ParticleSystem {
            render_type: ParticleRenderType::Billboard3D,
            max_particles: 10,
            //texture: ParticleTexture::Sprite(asset_server.load("px.png")),
            spawn_rate_per_second: 1000.0.into(),
            initial_speed: JitteredValue::jittered(2.0, -0.2..0.2),
            velocity_modifiers: vec![VelocityModifier::Drag(0.01.into())],
            lifetime: JitteredValue::jittered(8.0, -2.0..2.0),
            color: ColorOverTime::Gradient(Curve::new(vec![
                CurvePoint::new(Color::PURPLE, 0.0),
                CurvePoint::new(Color::RED, 0.5),
                CurvePoint::new(Color::rgba(0.0, 0.0, 1.0, 1.0), 1.0),
            ])),
            looping: true,
            system_duration_seconds: 10.0,
            max_distance: Some(12.0),
            scale: 10.0.into(),
            ..ParticleSystem::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..ParticleSystemBundle::default()
    })
    .insert(Playing);*/

    commands.spawn(ParticleSystemBundle {
        particle_system: ParticleSystem {
            render_type: ParticleRenderType::Billboard3D,
            max_particles: 50_000,
            //texture: ParticleTexture::Sprite(asset_server.load("px.png")),
            spawn_rate_per_second: 1000.0.into(),
            initial_speed: JitteredValue::jittered(2.0, -0.2..0.2),
            velocity_modifiers: vec![VelocityModifier::Drag(0.01.into())],
            lifetime: JitteredValue::jittered(8.0, -2.0..2.0),
            color: ColorOverTime::Gradient(Curve::new(vec![
                CurvePoint::new(Color::PURPLE, 0.0),
                CurvePoint::new(Color::RED, 0.5),
                CurvePoint::new(Color::rgba(0.0, 0.0, 1.0, 1.0), 1.0),
            ])),
            looping: true,
            system_duration_seconds: 10.0,
            max_distance: Some(12.0),
            scale: 0.1.into(),
            ..ParticleSystem::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..ParticleSystemBundle::default()
    })
    .insert(Playing);

    // cube
    /*commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 3.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(5.0, 5.0, 5.0),
        ..default()
    });*/

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(2.0, 2.5, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
