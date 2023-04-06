//! This example demonstrates the how velocity modifiers works:
//! The red particle system only has a Constant Acceleration that makes particles accelerate upwards over time.
//! The green one only has a Drag effect that makes particles slow down over time.
//! The blue one only has a Noise2D that affects randomly how particles are moving.
//! The orange one combine all three effects together.
//!
//! There is no limit but performance in how much velocity modifiers a particle system can have simultaneously.

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_particle_systems::{ParticleSystemPlugin, ParticleTexture, ParticleSystemBundle, CircleSegment, ParticleSystem, Playing};
use bevy_particle_systems::render::BillboardMaterial;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ParticleSystemPlugin::default()) // <-- Add the plugin
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut billboard_materials: ResMut<Assets<BillboardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(3.0, 5.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // load a texture and retrieve its aspect ratio
    let texture_handle = asset_server.load("arrow.png");

    // load custom billboard shader
    let billboard_material = billboard_materials.add(BillboardMaterial {
        color: Color::GREEN,
        size: Vec2::splat(1.0),
        texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
    });

    // Billboard plane
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0, subdivisions: 0 })),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        material: billboard_material,
        ..default()
    });

    // standard material
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    // Standard plane
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0, subdivisions: 0 })),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        material: material,
        ..default()
    });

    

    // Particle System
    commands.spawn(ParticleSystemBundle {
        particle_system: ParticleSystem {
            emitter_shape: CircleSegment {
                opening_angle: PI*2.0,
                ..Default::default()
            }
            .into(),
            render_type: bevy_particle_systems::ParticleRenderType::Billboard3D,
            spawn_rate_per_second: 10.0.into(),
            texture: ParticleTexture::Sprite(asset_server.load("px.png")),
            lifetime: 2.3.into(),
            system_duration_seconds: 10.0,
            initial_speed: (1.0..2.0).into(),
            scale: 1.0.into(),
            color: Color::WHITE.into(),
            ..default()
        },
        transform: Transform::from_xyz(1.5, 2.5, 0.0),
        ..Default::default()
    }).insert(Playing);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::WHITE,
            ..default()
        },
        transform: Transform::from_xyz(0.5, 1.0, 0.0),
        ..Default::default()
    });
}
