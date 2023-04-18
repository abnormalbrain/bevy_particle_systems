//! A shader that renders a mesh multiple times in one draw call.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_particle_systems::*;
use bevy_render::view::NoFrustumCulling;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ParticleSystemPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        //.add_startup_system(setup)
        //.add_system(update_particles)
        .add_system(setup)
        .run();
}

fn update_particles(
    mut ptcs: Query<(Entity, &mut ParticleSystemInstancedData)>,
) {
    ptcs.for_each_mut(|(entity, mut inst_data)| {
        for i in &mut inst_data.0 {
            i.1.position += Vec3::new(0.0, 0.01, 0.0);
        }

        inst_data.0.insert(
            entity,
            ParticleBillboardInstanceData {
                position: Vec3::splat(0.0),
                scale: 1.0,
                color: Color::RED.as_rgba_f32()
        });
    });
}

#[derive(Resource)]
pub struct IsCheck;

fn setup(
    mut commands: Commands,
    billboard_mesh: Res<BillboardMeshHandle>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    check: Option<Res<IsCheck>>,
    mut meshes: ResMut<Assets<Mesh>>,
    //asset_server: Res<AssetServer>,

) {
    if let Some(_) = check {
        return;
    } else {
        commands.insert_resource(IsCheck);
    }

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
        ..ParticleSystemBundle::default()
    })
    .insert(Playing);

    // light
    /*commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });*/

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(2.0, 2.5, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
