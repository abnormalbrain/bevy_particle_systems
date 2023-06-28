use crate::{
    components::{
        BurstIndex, Lifetime, Particle, ParticleBundle, ParticleColor, ParticleCount,
        ParticleSpace, ParticleSystem, Playing, RunningState, Velocity,
    },
    values::{ColorOverTime, PrecalculatedParticleVariables, VelocityModifier},
    AnimatedIndex, AtlasIndex, BillboardMeshHandle, DistanceTraveled, InstancedParticle, Lerpable,
    ParticleBillboardInstanceData, ParticleRenderType, ParticleSystemInstancedData,
    ParticleSystemInstancedDataBundle, ParticleTexture, SortParticleByDepth, VelocityAligned,
    VelocityAlignedType,
};
use bevy_asset::Handle;
use bevy_ecs::prelude::{Commands, Entity, Query, Res, SystemSet, With};
use bevy_hierarchy::BuildChildren;
use bevy_math::{Quat, Vec2, Vec3, Vec3Swizzles};
use bevy_render::{
    prelude::Image,
    view::{visibility::ComputedVisibility, NoFrustumCulling},
};
use bevy_sprite::prelude::{Sprite, SpriteBundle};
use bevy_sprite::{SpriteSheetBundle, TextureAtlasSprite};
use bevy_time::Time;
use bevy_transform::prelude::{GlobalTransform, Transform};
use std::collections::BTreeMap;
use std::f32::consts::PI;

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

        if let ParticleRenderType::Billboard3d(_) = particle_system.render_type {
            match &particle_system.texture {
                ParticleTexture::Sprite(image_handle) => {
                    commands.entity(entity).insert(image_handle.clone());
                }
                ParticleTexture::TextureAtlas { .. } => {
                    panic!("Particle System Error: Texture Atlas not supported for 3D billboard rendering!");
                }
                ParticleTexture::None => {
                    commands.entity(entity).remove::<Handle<Image>>();
                }
            }
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

            let initial_rotation = particle_system.initial_rotation.get_value(&mut rng);

            // Will be useful to determine the rotation of billboard 3D particles
            let mut world_alignment: Vec3 = Vec3::splat(0.0);

            // the Z axis must not be altered if we use 3D
            let is_2d = particle_system.render_type == crate::ParticleRenderType::Sprite2D;

            // depending on the rendering type, we wont use the same "forward" axis
            let direction = if is_2d {
                spawn_point.local_x()
            } else {
                spawn_point.forward()
            };

            if is_2d {
                spawn_point.translation.z = particle_system
                    .z_value_override
                    .as_ref()
                    .map_or(0.0, |jittered_value| jittered_value.get_value(&mut rng));

                if let Some(alignment) = &particle_system.align_with_velocity {
                    // The transform is already aligned on the velocity with the X axis
                    match alignment {
                        VelocityAlignedType::X => {}
                        VelocityAlignedType::NegativeX => {
                            let rotation = Quat::from_rotation_z(PI);
                            spawn_point.rotate(rotation);
                        }
                        VelocityAlignedType::Y => {
                            let rotation = Quat::from_rotation_z(PI * 0.5);
                            spawn_point.rotate(rotation);
                        }
                        VelocityAlignedType::NegativeY => {
                            let rotation = Quat::from_rotation_z(PI * 1.5);
                            spawn_point.rotate(rotation);
                        }
                        VelocityAlignedType::Z => {
                            panic!("Cannot align with Z axis with 2D particles");
                        }
                        VelocityAlignedType::NegativeZ => {
                            panic!("Cannot align with -Z axis with 2D particles");
                        }
                        VelocityAlignedType::CustomLocal(v) => {
                            let local_axis = spawn_point.rotation * *v;
                            let rotation = Quat::from_rotation_arc_2d(
                                spawn_point.local_x().xy(),
                                local_axis.xy(),
                            );
                            spawn_point.rotate(rotation);
                        }
                        VelocityAlignedType::CustomGlobal(v) => {
                            let rotation =
                                Quat::from_rotation_arc_2d(spawn_point.local_x().xy(), v.xy());
                            spawn_point.rotate(rotation);
                        }
                    };
                    // Then we apply the initial rotation (which is often 0)
                    if initial_rotation != 0.0 {
                        spawn_point.rotate_z(initial_rotation);
                    }
                } else {
                    // If no alignement required, we override the rotation with the provided initial_rotation
                    if initial_rotation != 0.0 {
                        spawn_point.rotation = Quat::from_rotation_z(initial_rotation);
                    }
                }
            } else if let Some(alignment) = &particle_system.align_with_velocity {
                world_alignment += alignment.get_billboard_alignment();
            }

            let particle_scale = particle_system.scale.at_lifetime_pct(0.0);
            spawn_point.scale = Vec3::new(particle_scale, particle_scale, particle_scale);

            let initial_speed = particle_system.initial_speed.get_value(&mut rng);
            let particle_velocity = direction * initial_speed;

            // Spawn the particle
            let mut particle_entity_commands = commands.spawn(ParticleBundle {
                particle: Particle {
                    parent_system: entity,
                    max_lifetime: particle_system.lifetime.get_value(&mut rng),
                    max_distance: particle_system.max_distance,
                    use_scaled_time: particle_system.use_scaled_time,
                    scale: particle_system.scale.clone(),
                    initial_rotation,
                    rotation_speed: particle_system.rotation_speed.get_value(&mut rng),
                    velocity_modifiers: particle_system.velocity_modifiers.clone(),
                    despawn_with_parent: particle_system.despawn_particles_with_system,
                },
                velocity: Velocity::new(particle_velocity, is_2d),
                distance: DistanceTraveled {
                    dist_squared: 0.0,
                    from: spawn_point.translation,
                },
                color: ParticleColor(particle_system.color.clone()),
                ..ParticleBundle::default()
            });
            if let Some(alignment) = &particle_system.align_with_velocity {
                particle_entity_commands.insert(VelocityAligned(alignment.clone()));
            }

            // If we use local space, then parent the particle to the particle system
            if let ParticleSpace::Local = particle_system.space {
                let particle_entity = particle_entity_commands.id();
                particle_entity_commands
                    .commands()
                    .entity(entity)
                    .push_children(&[particle_entity]);
            }

            match &particle_system.render_type {
                crate::ParticleRenderType::Sprite2D => match &particle_system.texture {
                    ParticleTexture::None => (),
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
                },
                crate::ParticleRenderType::Billboard3d(billboard_settings) => {
                    // Gather particle instance data
                    let particle_id = particle_entity_commands.id();
                    let particle_inst_data = ParticleBillboardInstanceData {
                        position: spawn_point.translation,
                        scale: particle_system.scale.clone().at_lifetime_pct(0.0),
                        rotation: initial_rotation,
                        velocity: particle_velocity,
                        alignment: world_alignment,
                        color: particle_system.color.at_lifetime_pct(0.0).as_rgba_f32(),
                    };

                    // Insert it into the ParticleSystemInstanceData of its owner particle system if there is one...
                    if let Some(ref mut inst_data) = instanced_data {
                        inst_data.0.insert(particle_id, particle_inst_data);
                    // ...Create the ParticleSystemInstanceData and insert the particle data if there isn't
                    } else {
                        let mut inst_data = BTreeMap::new();
                        inst_data.insert(particle_id, particle_inst_data);

                        let mut particle_system_commands =
                            particle_entity_commands.commands().entity(entity);

                        particle_system_commands.insert(ParticleSystemInstancedDataBundle {
                            mesh_handle: billboard_mesh.0.clone(),
                            computed_visibility: ComputedVisibility::default(),
                            inst_data: ParticleSystemInstancedData(inst_data),
                        });
                        // Marker component that disables frustrum culling
                        if !billboard_settings.use_frustrum_culling {
                            particle_system_commands.insert(NoFrustumCulling);
                        }
                        // Marker component that enables sorting particles by depth (useful for overlapping transparency)
                        if billboard_settings.sort_particles_by_depth {
                            particle_system_commands.insert(SortParticleByDepth);
                        }
                    };

                    particle_entity_commands
                        .insert(InstancedParticle(entity))
                        .insert(spawn_point);
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

#[allow(clippy::type_complexity)]
pub(crate) fn particle_transform(
    mut particle_query: Query<(
        &Particle,
        &Lifetime,
        &mut Velocity,
        &mut DistanceTraveled,
        &mut Transform,
        Option<&InstancedParticle>,
    )>,
    time: Res<Time>,
) {
    particle_query.par_iter_mut().for_each_mut(
        |(particle, lifetime, mut velocity, mut distance, mut transform, instanced_particle)| {
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
                use VelocityModifier::{Drag, Noise2D, Noise3D, Scalar, Vector};
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

                    Noise2D(n) => {
                        let offset = n.sample(
                            Vec2::new(transform.translation.x, transform.translation.y),
                            elapsed_time,
                        ) * delta_time;
                        velocity.0 += Vec3::new(offset.x, offset.y, 0.0);
                    }

                    Noise3D(n) => {
                        let offset = n.sample(transform.translation, elapsed_time) * delta_time;
                        velocity.0 += offset;
                    }
                }
            }

            // Apply velocity to translation
            transform.translation += velocity.0 * delta_time;
            // Apply scale
            transform.scale = Vec3::splat(particle.scale.at_lifetime_pct(lifetime_pct));

            // Apply rotation to the particle only if its not instanced as billboard
            // Otherwise, the rotation of the transform is ignored by the rendering pipeline
            if instanced_particle.is_none() {
                // Apply rotation
                transform.rotate_z(particle.rotation_speed * time.delta_seconds());
            }

            // Update distance travelled
            distance.dist_squared = transform.translation.distance_squared(distance.from);
        },
    );
}

#[allow(clippy::type_complexity)]
pub(crate) fn update_instanced_particles(
    particle_query: Query<
        (
            &Particle,
            &Transform,
            &ParticleColor,
            &Lifetime,
            Option<&VelocityAligned>,
        ),
        With<InstancedParticle>,
    >,
    mut inst_data_query: Query<Option<&mut ParticleSystemInstancedData>, With<ParticleSystem>>,
) {
    // Do only for each particle system with instanced data
    inst_data_query.for_each_mut(|inst_data| {
        if let Some(mut inst_data) = inst_data {
            for (&particle, instance) in &mut inst_data.0 {
                if let Ok((p, p_transform, p_color, p_lifetime, p_velocity_aligned)) =
                    particle_query.get(particle)
                {
                    instance.position = p_transform.translation;
                    instance.scale = p_transform.scale.x;
                    instance.rotation = p.initial_rotation + p.rotation_speed * p_lifetime.0;
                    instance.alignment =
                        if let Some(VelocityAligned(alignment)) = p_velocity_aligned {
                            alignment.get_billboard_alignment()
                        } else {
                            Vec3::ZERO
                        };
                    let pct = p_lifetime.0 / p.max_lifetime;
                    instance.color = p_color.0.at_lifetime_pct(pct).as_rgba_f32();
                }
            }
        }
    });
}

pub(crate) fn particle_cleanup(
    particle_query: Query<(
        Entity,
        &Particle,
        &Lifetime,
        &DistanceTraveled,
        Option<&InstancedParticle>,
    )>,
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
                &mut commands,
            );
        } else if particle.despawn_with_parent
            && commands.get_entity(particle.parent_system).is_none()
        {
            despawn_particle(
                entity,
                inst_particle,
                &mut instanced_data_query,
                &mut commands,
            );
        }
    }
}

fn despawn_particle(
    particle_entity: Entity,
    instance: Option<&InstancedParticle>,
    instanced_data_query: &mut Query<&mut ParticleSystemInstancedData>,
    commands: &mut Commands,
) {
    // remove the particle entry from the instanced data if needed
    if let Some(instance) = instance {
        instanced_data_query
            .get_mut(instance.0)
            .unwrap() // There should always be an entry corresponding to the current particle if it exists.
            .0
            .remove(&particle_entity);
    }

    // despawn the particle entity
    commands.entity(particle_entity).despawn();
}
