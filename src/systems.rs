use std::collections::BTreeMap;
use bevy_asset::Assets;
use bevy_ecs::prelude::{Commands, Entity, Query, Res, SystemSet, With, ResMut};
use bevy_hierarchy::BuildChildren;
use bevy_math::{Quat, Vec2, Vec3};
use bevy_sprite::prelude::{Sprite, SpriteBundle};
use bevy_sprite::{SpriteSheetBundle, TextureAtlasSprite};
use bevy_time::Time;
use bevy_transform::prelude::{GlobalTransform, Transform};
use bevy_render::{
    prelude::{SpatialBundle, Mesh, shape},
    color::Color,
    view::{NoFrustumCulling, visibility::ComputedVisibility},
};
use crate::BillboardMeshHandle;
use crate::{
    components::{
        BurstIndex, Lifetime, Particle, ParticleBundle, ParticleColor, ParticleCount,
        ParticleSpace, ParticleSystem, Playing, RunningState, Velocity,
    },
    values::{ColorOverTime, PrecalculatedParticleVariables, VelocityModifier},
    DistanceTraveled, ParticleTexture, AnimatedIndex, AtlasIndex, Lerpable,
    ParticleSystemInstancedData, ParticleBillboardInstanceData,
    ParticleSystemInstancedDataBundle, InstancedParticle
};

/// System label attached to the `SystemSet` provided in this plugin
///
/// This is provided so that users can order their systems to run before/after this plugin.
#[derive(Debug, SystemSet, Hash, Clone, PartialEq, Eq)]
pub struct ParticleSystemSet;

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
            Option<&mut ParticleSystemInstancedData>,
        ),
        With<Playing>,
    >,
    time: Res<Time>,
    mut commands: Commands,
    billboard_mesh: Res<BillboardMeshHandle>,
) {
    let mut rng = rand::thread_rng();
    for (
        entity,
        global_transform,
        particle_system,
        mut particle_count,
        mut running_state,
        mut burst_index,
        mut instanced_data,
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
            .clamp(0.0, remaining_particles) as usize;

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
            let origin_pos = match particle_system.space {
                ParticleSpace::Local => Transform::default(),
                ParticleSpace::World => Transform::from(*global_transform),
            };

            let spawn_pos = particle_system.emitter_shape.sample(&mut rng);

            let mut spawn_point = origin_pos.mul_transform(spawn_pos);

            let direction = spawn_point.rotation * Vec3::X;

            spawn_point.translation.z = particle_system
                .z_value_override
                .as_ref()
                .map_or(0.0, |jittered_value| jittered_value.get_value(&mut rng));
            let particle_scale = particle_system.scale.at_lifetime_pct(0.0);
            spawn_point.scale = Vec3::new(particle_scale, particle_scale, particle_scale);

            if particle_system.rotate_to_movement_direction {
                spawn_point.rotate_z(particle_system.initial_rotation.get_value(&mut rng));
            } else {
                spawn_point.rotation =
                    Quat::from_rotation_z(particle_system.initial_rotation.get_value(&mut rng));
            };

            // Spawn the particle
            let mut particle_entity_commands = commands.spawn(ParticleBundle {
                particle: Particle {
                    parent_system: entity,
                    max_lifetime: particle_system.lifetime.get_value(&mut rng),
                    max_distance: particle_system.max_distance,
                    use_scaled_time: particle_system.use_scaled_time,
                    scale: particle_system.scale.clone(),
                    rotation_speed: particle_system.rotation_speed.get_value(&mut rng),
                    velocity_modifiers: particle_system.velocity_modifiers.clone(),
                    despawn_with_parent: particle_system.despawn_particles_with_system,
                },
                velocity: Velocity::new(
                    direction * particle_system.initial_speed.get_value(&mut rng),
                    true,
                ),
                distance: DistanceTraveled {
                    dist_squared: 0.0,
                    from: spawn_point.translation,
                },
                color: ParticleColor(particle_system.color.clone()),
                ..ParticleBundle::default()
            });

            // If we use local space, then parent the particle to the particle system
            if let ParticleSpace::Local = particle_system.space {
                let particle_entity = particle_entity_commands.id();
                particle_entity_commands
                    .commands()
                    .entity(entity)
                    .push_children(&[particle_entity]);
            }

            match &particle_system.render_type {
                crate::ParticleRenderType::Sprite2D => {
                    match &particle_system.texture {
                        ParticleTexture::Sprite(image_handle) => {
                            particle_entity_commands.insert(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: particle_system.rescale_texture,
                                    color: particle_system.color.at_lifetime_pct(0.0),
                                    ..Sprite::default()
                                },
                                transform: spawn_point,
                                texture: image_handle.clone(),
                                ..SpriteBundle::default()
                            });
                        },
                        ParticleTexture::TextureAtlas {
                            atlas: atlas_handle,
                            index,
                        } => {
                            particle_entity_commands.insert(SpriteSheetBundle {
                                sprite: TextureAtlasSprite {
                                    custom_size: particle_system.rescale_texture,
                                    color: particle_system.color.at_lifetime_pct(0.0),
                                    index: index.get_value(&mut rng),
                                    ..TextureAtlasSprite::default()
                                },
                                transform: spawn_point,
                                texture_atlas: atlas_handle.clone(),
                                ..SpriteSheetBundle::default()
                            });
        
                            if let AtlasIndex::Animated(animated_index) = index {
                                particle_entity_commands.insert(animated_index.clone());
                            };
                        }
                    }
                },
                crate::ParticleRenderType::Billboard3D => {
                    let particle_id = particle_entity_commands.id();
                    let particle_inst_data = ParticleBillboardInstanceData {
                        position: spawn_point.translation,
                        scale: particle_system.scale.clone().at_lifetime_pct(0.0),
                        color: particle_system.color.at_lifetime_pct(0.0).as_rgba_f32(),
                    };

                    if let Some(ref mut inst_data) = instanced_data {
                        inst_data.0.insert(
                            particle_id,
                            particle_inst_data);
                    } else {
                        let mut inst_data = BTreeMap::new();
                        inst_data.insert(
                            particle_id,
                            particle_inst_data);
                        particle_entity_commands
                            .commands()
                            .entity(entity)
                            .insert(ParticleSystemInstancedDataBundle {
                                mesh_handle: billboard_mesh.0.clone(),
                                computed_visibility: ComputedVisibility::default(),
                                inst_data: ParticleSystemInstancedData(inst_data),
                                disabled_frustrum_culling: NoFrustumCulling,
                        });
                    };
                    
                    particle_entity_commands.insert(InstancedParticle(entity));
                },
            }

            match &particle_system.texture {
                ParticleTexture::Sprite(image_handle) => {
                    particle_entity_commands.insert(SpriteBundle {
                        sprite: Sprite {
                            custom_size: particle_system.rescale_texture,
                            color: particle_system.color.at_lifetime_pct(0.0),
                            ..Sprite::default()
                        },
                        transform: spawn_point,
                        texture: image_handle.clone(),
                        ..SpriteBundle::default()
                    });
                }
                ParticleTexture::TextureAtlas {
                    atlas: atlas_handle,
                    index,
                } => {
                    particle_entity_commands.insert(SpriteSheetBundle {
                        sprite: TextureAtlasSprite {
                            custom_size: particle_system.rescale_texture,
                            color: particle_system.color.at_lifetime_pct(0.0),
                            index: index.get_value(&mut rng),
                            ..TextureAtlasSprite::default()
                        },
                        transform: spawn_point,
                        texture_atlas: atlas_handle.clone(),
                        ..SpriteSheetBundle::default()
                    });

                    if let AtlasIndex::Animated(animated_index) = index {
                        particle_entity_commands.insert(animated_index.clone());
                    };
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
    lifetime_query
        .par_iter_mut()
        .for_each_mut(|(mut lifetime, particle)| {
            if particle.use_scaled_time {
                lifetime.0 += time.delta_seconds();
            } else {
                lifetime.0 += time.raw_delta_seconds();
            }
        });
}

pub(crate) fn particle_sprite_color(
    mut particle_query: Query<(&Particle, &mut ParticleColor, &Lifetime, &mut Sprite)>,
) {
    particle_query.par_iter_mut().for_each_mut(
        |(particle, mut particle_color, lifetime, mut sprite)| {
            let pct = lifetime.0 / particle.max_lifetime;
            sprite.color = match &mut particle_color.0 {
                ColorOverTime::Constant(color) => *color,
                ColorOverTime::Lerp(lerp) => lerp.a.lerp(lerp.b, pct),
                ColorOverTime::Gradient(curve) => curve.sample_mut(pct),
            };
        },
    );
}

pub(crate) fn particle_texture_atlas_color(
    mut particle_query: Query<(
        &Particle,
        &mut ParticleColor,
        &Lifetime,
        &mut TextureAtlasSprite,
        Option<&AnimatedIndex>,
    )>,
) {
    particle_query.par_iter_mut().for_each_mut(
        |(particle, mut particle_color, lifetime, mut sprite, anim_index)| {
            let pct = lifetime.0 / particle.max_lifetime;
            sprite.color = match &mut particle_color.0 {
                ColorOverTime::Constant(color) => *color,
                ColorOverTime::Lerp(lerp) => lerp.a.lerp(lerp.b, pct),
                ColorOverTime::Gradient(curve) => curve.sample_mut(pct),
            };

            if let Some(anim_index) = anim_index {
                sprite.index = anim_index.get_at_time(lifetime.0);
            }
        },
    );
}

pub(crate) fn particle_transform(
    mut particle_query: Query<(
        &Particle,
        &Lifetime,
        &mut Velocity,
        &mut DistanceTraveled,
        &mut Transform,
    )>,
    time: Res<Time>,
) {
    particle_query.par_iter_mut().for_each_mut(
        |(particle, lifetime, mut velocity, mut distance, mut transform)| {
            let lifetime_pct = lifetime.0 / particle.max_lifetime;

            let (delta_time, elapsed_time) = if particle.use_scaled_time {
                (time.delta_seconds(), time.elapsed_seconds_wrapped())
            } else {
                (time.raw_delta_seconds(), time.raw_elapsed_seconds_wrapped())
            };

            // inititalize precalculated values
            let mut ppv = PrecalculatedParticleVariables::new();

            // Apply velocity modifiers to velocity
            for modifier in &particle.velocity_modifiers {
                use VelocityModifier::{Drag, Noise, Scalar, Vector};
                match modifier {
                    Vector(v) => {
                        velocity.0 += v.at_lifetime_pct(lifetime_pct) * delta_time;
                    }

                    Scalar(v) => {
                        let direction = ppv.get_particle_direction(&velocity.0);
                        velocity.0 += v.at_lifetime_pct(lifetime_pct) * direction * delta_time;
                    }

                    Drag(v) => {
                        let current_drag = v.at_lifetime_pct(lifetime_pct);
                        if current_drag > 0.0 {
                            let drag_force =
                                ppv.get_particle_sqr_speed(&velocity.0) * current_drag * delta_time;
                            let direction = ppv.get_particle_direction(&velocity.0);
                            velocity.0 -= direction * drag_force;
                        }
                    }

                    Noise(n) => {
                        let offset = n.sample(
                            Vec2::new(transform.translation.x, transform.translation.y),
                            elapsed_time,
                        ) * delta_time;
                        velocity.0 += Vec3::new(offset.x, offset.y, 0.0);
                    }
                }
            }
            transform.translation += velocity.0 * delta_time;

            transform.scale = Vec3::splat(particle.scale.at_lifetime_pct(lifetime_pct));
            transform.rotate_z(particle.rotation_speed * time.delta_seconds());

            distance.dist_squared = transform.translation.distance_squared(distance.from);
        },
    );
}

pub(crate) fn update_instanced_particles(
    particle_query: Query<(&Particle, &Transform, &ParticleColor, &Lifetime), With<InstancedParticle>>,
    mut inst_data_query: Query<Option<&mut ParticleSystemInstancedData>, With<ParticleSystem>>,
) {
    // Do only for each particle system with instanced data
    inst_data_query.for_each_mut( |inst_data| {
        if let Some(mut inst_data) = inst_data {
            for (particle, instance) in inst_data.0.iter_mut() {
                if let Ok((p, p_transform, p_color, p_lifetime)) = particle_query.get(*particle) {
                    instance.position = p_transform.translation;
                    instance.scale = p_transform.scale.x;
                    let pct = p_lifetime.0 / p.max_lifetime;
                    instance.color = p_color.0.at_lifetime_pct(pct).as_rgba_f32();
                }
            }
        }
    });
}

pub(crate) fn particle_cleanup(
    particle_query: Query<(Entity, &Particle, &Lifetime, &DistanceTraveled, Option<&InstancedParticle>)>,
    mut particle_count_query: Query<&mut ParticleCount>,
    mut instanced_data_query: Query<&mut ParticleSystemInstancedData>,
    mut commands: Commands,
) {
    for (entity, particle, lifetime, distance, inst_particle) in particle_query.iter() {
        if lifetime.0 >= particle.max_lifetime
            || (particle.max_distance.is_some()
                && distance.dist_squared >= particle.max_distance.unwrap().powi(2))
        {
            if let Ok(mut particle_count) = particle_count_query.get_mut(particle.parent_system) {
                if particle_count.0 > 0 {
                    particle_count.0 -= 1;
                }
            }
            despawn_particle(
                entity,
                inst_particle,
                &mut instanced_data_query,
                &mut commands
            );
        } else if particle.despawn_with_parent
            && commands.get_entity(particle.parent_system).is_none()
        {
            despawn_particle(
                entity,
                inst_particle,
                &mut instanced_data_query,
                &mut commands
            );
        }
    }
}

fn despawn_particle(
    particle: Entity,
    instance: Option<&InstancedParticle>,
    instanced_data_query: &mut Query<&mut ParticleSystemInstancedData>,
    commands: &mut Commands,
) {
    // remove the particle from the instanced data if needed
    if let Some(instance) = instance {
        if let Ok(mut instanced_data) = instanced_data_query.get_mut(instance.0) {
            instanced_data.0.remove(&particle);
        } else {
            panic!("This is not supposed to happen");
        }
    }
    // despawn the particle entity
    commands.entity(particle).despawn();
}

pub(crate) fn setup_billboard_resource(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let handle = meshes.add(Mesh::from(shape::Plane {
        size: -0.5,
        subdivisions: 0,
    }));
    commands.insert_resource(BillboardMeshHandle(handle));
}