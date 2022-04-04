use bevy::prelude::Plugin;

use crate::systems::{
    partcle_spawner, particle_cleanup, particle_color, particle_lifetime, particle_transform,
};

#[derive(Default)]
pub struct ParticleSystemPlugin;

impl Plugin for ParticleSystemPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(partcle_spawner)
            .add_system(particle_lifetime)
            .add_system(particle_color)
            .add_system(particle_transform)
            .add_system(particle_cleanup);
    }
}
