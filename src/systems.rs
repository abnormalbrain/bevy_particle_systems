use bevy_asset::Handle;
use bevy_ecs::prelude::{Commands, Entity, Query, Res, With};
use bevy_hierarchy::BuildChildren;
use bevy_math::Vec3;
use bevy_render::prelude::Image;
use bevy_sprite::prelude::{Sprite, SpriteBundle};
use bevy_sprite::{SpriteSheetBundle, TextureAtlasSprite};
use bevy_time::Time;
use bevy_transform::prelude::{GlobalTransform, Transform};
use rand::prelude::*;

use crate::{
    components::{
        BurstIndex, Direction, Lifetime, Particle, ParticleBundle, ParticleCount, ParticleSpace,
        ParticleSystem, Playing, RunningState, Speed,
    },
    values::ColorOverTime,
    DistanceTraveled,
};

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::type_complexity,
    clippy::too_many_lines
)]
pub fn particle_spawner(
    mut particle_systems: Query<
        (
            Entity,
            &GlobalTransform,
            &ParticleSystem,
            &mut ParticleCount,
            &mut RunningState,
            &mut BurstIndex,
        ),
        With<Playing>,
    >,
    time: Res<Time>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    for (
        entity,
        global_transform,
        particle_system,
        mut particle_count,
        mut running_state,
        mut burst_index,
    ) in particle_systems.iter_mut()
    {
        if particle_system.use_scaled_time {
            running_state.running_time += time.delta_seconds();
        } else {
            running_state.running_time += time.raw_delta_seconds();
        }

        if running_state.running_time.floor() > running_state.current_second + 0.5 {
            running_state.current_second = running_state.running_time.floor();
            running_state.spawned_this_second = 0;
        }

        if running_state.running_time >= particle_system.system_duration_seconds {
            if particle_system.looping {
                running_state.running_time -= particle_system.system_duration_seconds;
                running_state.current_second = running_state.running_time.floor();
                running_state.spawned_this_second = 0;
                burst_index.0 = 0;
            } else {
                if particle_count.0 == 0 {
                    if particle_system.despawn_on_finish {
                        commands.entity(entity).despawn();
                    } else {
                        commands.entity(entity).remove::<Playing>();
                    }
                }
                continue;
            }
        }

        if particle_count.0 >= particle_system.max_particles {
            continue;
        }

        let pct = running_state.running_time / particle_system.system_duration_seconds;
        let remaining_particles = (particle_system.max_particles - particle_count.0) as f32;
        let current_spawn_rate = particle_system.spawn_rate_per_second.at_lifetime_pct(pct);
        let mut to_spawn = ((running_state.running_time - running_state.running_time.floor())
            * current_spawn_rate
            - running_state.spawned_this_second as f32)
            .floor()
            .min(remaining_particles)
            .max(0.0) as usize;

        let mut extra = 0;
        if !particle_system.bursts.is_empty() {
            if let Some(current_burst) = particle_system.bursts.get(burst_index.0) {
                if running_state.running_time >= current_burst.time {
                    extra += current_burst.count;
                    burst_index.0 += 1;
                }
            }
        }
        if to_spawn == 0
            && running_state.spawned_this_second == 0
            && particle_count.0 < particle_system.max_particles
            && current_spawn_rate > 0.0
        {
            to_spawn = 1;
        }

        if to_spawn == 0 && extra == 0 {
            continue;
        }

        for _ in 0..to_spawn + extra {
            let mut spawn_point = match particle_system.space {
                ParticleSpace::Local => Transform::default(),
                ParticleSpace::World => Transform::from(*global_transform),
            };
            let radian: f32 = rng.gen_range(-0.5..0.5) * particle_system.emitter_shape
                + particle_system.emitter_angle;
            let direction = Vec3::new(radian.cos(), radian.sin(), 0.0);

            spawn_point.translation += direction * particle_system.spawn_radius.get_value(&mut rng);
            spawn_point.translation.z = particle_system
                .z_value_override
                .as_ref()
                .map_or(0.0, |jittered_value| jittered_value.get_value(&mut rng));
            let particle_scale = particle_system.scale.at_lifetime_pct(0.0);
            spawn_point.scale = Vec3::new(particle_scale, particle_scale, particle_scale);

            match particle_system.space {
                ParticleSpace::World => {
                    let mut entity_commands = commands
                        .spawn(ParticleBundle {
                            particle: Particle {
                                parent_system: entity,
                                max_lifetime: particle_system.lifetime.get_value(&mut rng),
                                max_distance: particle_system.max_distance,
                                use_scaled_time: particle_system.use_scaled_time,
                                color: particle_system.color.clone(),
                                scale: particle_system.scale.clone(),
                                acceleration: particle_system.acceleration.clone(),
                                despawn_with_parent: particle_system.despawn_particles_with_system,
                            },
                            speed: Speed(particle_system.initial_speed.get_value(&mut rng)),
                            direction: Direction::new(
                                direction,
                                particle_system.z_value_override.is_some(),
                            ),
                            ..ParticleBundle::default()
                        });

                    if Some(image_handle) = particle_system.default_sprite.as_ref() {
                        entity_commands.insert(
                            SpriteBundle {
                                sprite: Sprite {
                                    color: particle_system.color.at_lifetime_pct(0.0),
                                    ..Sprite::default()
                                },
                                transform: spawn_point,
                                texture: image_handle.clone(),
                                ..SpriteBundle::default()
                            }
                        );
                    }
                    if Some(atlas_handle) = particle_system.texture_atlas.as_ref() {
                        if Some(atlas_index) = particle_system.texture_atlas_index {
                            entity_commands.insert(
                                SpriteSheetBundle {
                                    sprite: TextureAtlasSprite {
                                        color: particle_system.color.at_lifetime_pct(0.0),
                                        index: atlas_index,
                                        ..TextureAtlasSprite::default()
                                    },
                                    transform: spawn_point,
                                    texture_atlas: atlas_handle.clone(),
                                    ..SpriteSheetBundle::default()
                                }
                            );
                        }
                    }
                }
                ParticleSpace::Local => {
                    commands.entity(entity).with_children(|parent| {
                        let mut entity_commands = parent
                            .spawn(ParticleBundle {
                                particle: Particle {
                                    parent_system: entity,
                                    max_lifetime: particle_system.lifetime.get_value(&mut rng),
                                    max_distance: particle_system.max_distance,
                                    use_scaled_time: particle_system.use_scaled_time,
                                    color: particle_system.color.clone(),
                                    scale: particle_system.scale.clone(),
                                    acceleration: particle_system.acceleration.clone(),
                                    despawn_with_parent: particle_system
                                        .despawn_particles_with_system,
                                },
                                speed: Speed(particle_system.initial_speed.get_value(&mut rng)),
                                direction: Direction::new(
                                    direction,
                                    particle_system.z_value_override.is_some(),
                                ),
                                ..ParticleBundle::default()
                            });

                        if Some(image_handle) = particle_system.default_sprite.as_ref() {
                            entity_commands.insert(
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: particle_system.color.at_lifetime_pct(0.0),
                                        ..Sprite::default()
                                    },
                                    transform: spawn_point,
                                    texture: image_handle.clone(),
                                    ..SpriteBundle::default()
                                }
                            );
                        }
                        if Some(atlas_handle) = particle_system.texture_atlas.as_ref() {
                            if Some(atlas_index) = particle_system.texture_atlas_index {
                                entity_commands.insert(
                                    SpriteSheetBundle {
                                        sprite: TextureAtlasSprite {
                                            color: particle_system.color.at_lifetime_pct(0.0),
                                            index: atlas_index,
                                            ..TextureAtlasSprite::default()
                                        },
                                        transform: spawn_point,
                                        texture_atlas: atlas_handle.clone(),
                                        ..SpriteSheetBundle::default()
                                    }
                                );
                            }
                        }
                    });
                }
            }
        }
        // Don't count bursts in the normal spawn rate, but still count them in the particle cap.
        running_state.spawned_this_second += to_spawn;
        particle_count.0 += to_spawn + extra;
    }
}

pub(crate) fn particle_lifetime(
    mut lifetime_query: Query<(&mut Lifetime, &Particle)>,
    time: Res<Time>,
) {
    lifetime_query.par_for_each_mut(512, |(mut lifetime, particle)| {
        if particle.use_scaled_time {
            lifetime.0 += time.delta_seconds();
        } else {
            lifetime.0 += time.raw_delta_seconds();
        }
    });
}

pub(crate) fn particle_color(mut particle_query: Query<(&Particle, &Lifetime, &mut Sprite)>) {
    particle_query.par_for_each_mut(512, |(particle, lifetime, mut sprite)| {
        match &particle.color {
            ColorOverTime::Constant(color) => sprite.color = *color,
            ColorOverTime::Gradient(gradient) => {
                let pct = lifetime.0 / particle.max_lifetime;
                sprite.color = gradient.get_color(pct);
            }
        }
    });
}

pub(crate) fn particle_transform(
    mut particle_query: Query<(
        &Particle,
        &Lifetime,
        &Direction,
        &mut DistanceTraveled,
        &mut Speed,
        &mut Transform,
    )>,
    time: Res<Time>,
) {
    particle_query.par_for_each_mut(
        512,
        |(particle, lifetime, direction, mut distance, mut speed, mut transform)| {
            let lifetime_pct = lifetime.0 / particle.max_lifetime;
            let initial_position = transform.translation;
            if particle.use_scaled_time {
                speed.0 +=
                    particle.acceleration.at_lifetime_pct(lifetime_pct) * time.delta_seconds();
                transform.translation += direction.0 * speed.0 * time.delta_seconds();
            } else {
                speed.0 +=
                    particle.acceleration.at_lifetime_pct(lifetime_pct) * time.raw_delta_seconds();
                transform.translation += direction.0 * speed.0 * time.raw_delta_seconds();
            }

            transform.scale = Vec3::splat(particle.scale.at_lifetime_pct(lifetime_pct));

            distance.0 += transform.translation.distance(initial_position);
        },
    );
}

pub(crate) fn particle_cleanup(
    particle_query: Query<(Entity, &Particle, &Lifetime, &DistanceTraveled)>,
    mut particle_count_query: Query<&mut ParticleCount>,
    mut commands: Commands,
) {
    for (entity, particle, lifetime, distance) in particle_query.iter() {
        if lifetime.0 >= particle.max_lifetime
            || (particle.max_distance.is_some() && distance.0 >= particle.max_distance.unwrap())
        {
            if let Ok(mut particle_count) = particle_count_query.get_mut(particle.parent_system) {
                if particle_count.0 > 0 {
                    particle_count.0 -= 1;
                }
            }
            commands.entity(entity).despawn();
        } else if particle.despawn_with_parent
            && commands.get_entity(particle.parent_system).is_none()
        {
            commands.entity(entity).despawn();
        }
    }
}
