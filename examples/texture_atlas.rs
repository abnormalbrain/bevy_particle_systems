//! This example demonstrates the texture atlas works:
//! [`TextureAtlas`] index can be animated, constant, or randomly choosen.
//! Here we use the same sprite as this Bevy example: https://bevyengine.org/examples/2d/sprite-sheet/
//!
//! For a constant index, use "2.into()". For a randomly choosen index, use "vec![2, 3, 10].into()"

use bevy::{
    prelude::{Camera2dBundle, ClearColor, Color, Commands, ImagePlugin, Res, ResMut, Transform},
    DefaultPlugins,
};
use bevy_app::{App, PluginGroup};
use bevy_asset::{AssetServer, Assets};
use bevy_math::Vec2;
use bevy_particle_systems::{
    CircleSegment, ColorOverTime, Curve, CurvePoint, ParticleSystem, ParticleSystemBundle,
    ParticleSystemPlugin, ParticleTexture, Playing,
};
use bevy_sprite::{Sprite, SpriteBundle, TextureAtlas};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(ParticleSystemPlugin)
        .add_startup_systems((startup_system, setup_ground))
        .run();
}

fn startup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let projectiles = asset_server.load("gabe-idle-run.png");
    let particle_atlas = atlases.add(TextureAtlas::from_grid(
        projectiles,
        Vec2::new(24.0, 24.0),
        7,
        1,
        None,
        None,
    ));
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                emitter_shape: CircleSegment {
                    opening_angle: 0.0,
                    ..Default::default()
                }
                .into(),
                spawn_rate_per_second: 4.0.into(),
                // Here we tell the atlas to ignore the first frame (which is not part of the run animation loop)
                // And we display every frame for 0.1 second
                texture: ParticleTexture::TextureAtlas {
                    atlas: particle_atlas,
                    index: (1..7, 0.1).into(),
                },
                lifetime: 2.3.into(),
                system_duration_seconds: 10.0,
                initial_speed: (150.0..250.0).into(),
                scale: 8.5.into(),
                color: ColorOverTime::Gradient(Curve::new(vec![
                    CurvePoint::new(Color::rgba(1.0, 1.0, 1.0, 0.0), 0.0),
                    CurvePoint::new(Color::WHITE, 0.1),
                    CurvePoint::new(Color::WHITE, 0.9),
                    CurvePoint::new(Color::rgba(1.0, 1.0, 1.0, 0.0), 1.0),
                ])),
                ..Default::default()
            },
            transform: Transform::from_xyz(-500.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(Playing);
}

// add the grey ground
fn setup_ground(mut commands: Commands) {
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::DARK_GRAY,
            custom_size: Some(Vec2 { x: 1000.0, y: 40.0 }),
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, -100.0, 0.0),
        ..Default::default()
    });
}
