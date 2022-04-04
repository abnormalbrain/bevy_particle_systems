use bevy::{
    core::Time,
    math::Vec3,
    prelude::{BuildChildren, Commands, Entity, Query, Res, Transform, With},
    sprite::{Sprite, SpriteBundle},
    tasks::ComputeTaskPool,
};
use rand::prelude::*;

use crate::{
    components::{
        BurstIndex, Direction, Lifetime, Particle, ParticleBundle, ParticleCount, ParticleSpace,
        ParticleSystem, Playing, RunningState, Velocity,
    },
    values::ColorOverTime,
};

pub fn partcle_spawner(
    mut particle_systems: Query<
        (
            Entity,
            &Transform,
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
        transform,
        particle_system,
        mut particle_count,
        mut running_state,
        mut burst_index,
    ) in particle_systems.iter_mut()
    {
        running_state.running_time += time.delta_seconds();

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
                    commands.entity(entity).despawn();
                }
                continue;
            }
        }

        if particle_count.0 >= particle_system.max_particles {
            continue;
        }

        let pct = running_state.running_time / particle_system.system_duration_seconds;
        let remaining_particles = (particle_system.max_particles - particle_count.0) as f32;

        let mut to_spawn = ((running_state.running_time - running_state.running_time.floor())
            * particle_system.spawn_rate_per_second.at_lifetime_pct(pct)
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
        {
            to_spawn = 1;
        }

        if to_spawn == 0 && extra == 0 {
            continue;
        }

        for _ in 0..to_spawn + extra {
            let mut spawn_point = *transform;
            let radian: f32 = rng.gen_range(0.0..1.0) * particle_system.emitter_shape
                + particle_system.emitter_angle;
            let direction = Vec3::new(radian.cos(), radian.sin(), 0.0);

            spawn_point.translation += direction * particle_system.spawn_radius.get_value(&mut rng);
            spawn_point.translation.z = particle_system
                .z_value_override
                .as_ref()
                .map_or(0.0, |jittered_value| jittered_value.get_value(&mut rng));
                
            match particle_system.space {
                ParticleSpace::World => {
                    commands
                        .spawn_bundle(ParticleBundle {
                            particle: Particle {
                                parent_system: entity,
                                max_lifetime: particle_system.lifetime.get_value(&mut rng),
                            },
                            velocity: Velocity(
                                particle_system.initial_velocity.get_value(&mut rng),
                            ),
                            direction: Direction::new(
                                direction,
                                particle_system.z_value_override.is_some(),
                            ),
                            ..ParticleBundle::default()
                        })
                        .insert_bundle(SpriteBundle {
                            transform: spawn_point,
                            texture: particle_system.default_sprite.clone(),
                            ..SpriteBundle::default()
                        });
                }
                ParticleSpace::Local => {
                    commands.entity(entity).with_children(|parent| {
                        parent
                            .spawn_bundle(ParticleBundle {
                                particle: Particle {
                                    parent_system: entity,
                                    max_lifetime: particle_system.lifetime.get_value(&mut rng),
                                },
                                velocity: Velocity(
                                    particle_system.initial_velocity.get_value(&mut rng),
                                ),
                                direction: Direction::new(
                                    direction,
                                    particle_system.z_value_override.is_some(),
                                ),
                                ..ParticleBundle::default()
                            })
                            .insert_bundle(SpriteBundle {
                                transform: spawn_point,
                                texture: particle_system.default_sprite.clone(),
                                ..SpriteBundle::default()
                            });
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
    mut lifetime_query: Query<&mut Lifetime>,
    time: Res<Time>,
    compute_task_pool: Res<ComputeTaskPool>,
) {
    lifetime_query.par_for_each_mut(&compute_task_pool, 512, |mut lifetime| {
        lifetime.0 += time.delta_seconds();
    });
}

pub(crate) fn particle_color(
    mut particle_query: Query<(&Particle, &Lifetime, &mut Sprite)>,
    particle_system_query: Query<&ParticleSystem>,
    compute_task_pool: Res<ComputeTaskPool>,
) {
    particle_query.par_for_each_mut(
        &compute_task_pool,
        512,
        |(particle, lifetime, mut sprite)| {
            if let Ok(particle_system) = particle_system_query.get(particle.parent_system) {
                match &particle_system.color {
                    ColorOverTime::Constant(color) => sprite.color = *color,
                    ColorOverTime::Gradient(gradient) => {
                        let pct = lifetime.0 / particle.max_lifetime;
                        sprite.color = gradient.get_color(pct);
                    }
                }
            }
        },
    );
}

pub(crate) fn particle_transform(
    mut particle_query: Query<(
        &Particle,
        &Lifetime,
        &Direction,
        &mut Velocity,
        &mut Transform,
    )>,
    particle_system_query: Query<&ParticleSystem>,
    time: Res<Time>,
    compute_task_pool: Res<ComputeTaskPool>,
) {
    particle_query.par_for_each_mut(
        &compute_task_pool,
        512,
        |(particle, lifetime, direction, mut velocity, mut transform)| {
            if let Ok(particle_system) = particle_system_query.get(particle.parent_system) {
                let lifetime_pct = lifetime.0 / particle.max_lifetime;
                velocity.0 += particle_system.acceleration.at_lifetime_pct(lifetime_pct);
                transform.translation += direction.0 * velocity.0 * time.delta_seconds();
                transform.scale = Vec3::splat(particle_system.scale.at_lifetime_pct(lifetime_pct));
            }
        },
    )
}

pub(crate) fn particle_cleanup(
    particle_query: Query<(Entity, &Particle, &Lifetime)>,
    mut particle_count_query: Query<&mut ParticleCount>,
    mut commands: Commands,
) {
    for (entity, particle, lifetime) in particle_query.iter() {
        if lifetime.0 >= particle.max_lifetime {
            if let Ok(mut particle_count) = particle_count_query.get_mut(particle.parent_system) {
                if particle_count.0 > 0 {
                    particle_count.0 -= 1;
                }
            }
            commands.entity(entity).despawn();
        }
    }
}
