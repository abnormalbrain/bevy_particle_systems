use bevy::{
    prelude::{Camera2dBundle, ClearColor, Color, Commands, Res, ResMut},
    DefaultPlugins,
};
use bevy_app::App;
use bevy_asset::{AssetServer, Assets};
use bevy_math::Vec2;
use bevy_particle_systems::{
    ColorOverTime, ColorPoint, Gradient, JitteredValue, ParticleSystem, ParticleSystemBundle,
    ParticleSystemPlugin, ParticleTexture, Playing,
};
use bevy_sprite::TextureAtlas;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugin(ParticleSystemPlugin)
        .add_startup_system(startup_system)
        .run();
}

fn startup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let projectiles = asset_server.load("projectiles.png");
    let particle_atlas = atlases.add(TextureAtlas::from_grid(
        projectiles,
        Vec2::new(32.0, 32.0),
        5,
        8,
        None,
        None,
    ));
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(ParticleSystemBundle {
            particle_system: ParticleSystem {
                texture: ParticleTexture::TextureAtlas {
                    atlas: particle_atlas,
                    index: Vec::from([10, 11, 12, 13, 15, 16, 17, 18]).into(),
                },
                lifetime: 3.0.into(),
                initial_speed: JitteredValue::jittered(150.0, -50.0..50.0),
                scale: 1.5.into(),
                color: ColorOverTime::Gradient(Gradient::new(vec![
                    ColorPoint::new(Color::WHITE, 0.0),
                    ColorPoint::new(Color::rgba(1.0, 1.0, 1.0, 0.0), 1.0),
                ])),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Playing);
}
