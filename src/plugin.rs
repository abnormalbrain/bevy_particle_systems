use bevy::prelude::Plugin;

use crate::{
    systems::{
        partcle_spawner, particle_cleanup, particle_color, particle_lifetime, particle_transform,
    },
    TimeScale,
};

