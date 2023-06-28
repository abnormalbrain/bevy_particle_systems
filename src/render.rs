//! Defines Data and methods used for rendering the particles.

use std::{collections::BTreeMap, cmp::Ordering};
use bevy_asset::{Handle, AssetServer, Assets};
use bevy_math::Vec3;
use bevy_app::{App, Plugin};
use bevy_render::{
    prelude::{Msaa, shape, Mesh},
    extract_component::{ ExtractComponentPlugin, ExtractComponent},
    mesh::{GpuBufferInfo, MeshVertexBufferLayout},
    render_phase::{
        AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand,
        RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass,
    },
    view::{ExtractedView, visibility::ComputedVisibility, ViewTarget},
    render_resource::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
        BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
        BindingType, BlendState, Buffer, BufferInitDescriptor, BufferUsages,
        ColorTargetState, ColorWrites, PipelineCache, RenderPipelineDescriptor,
        SamplerBindingType, Shader, ShaderStages, SpecializedMeshPipeline,
        SpecializedMeshPipelineError, SpecializedMeshPipelines,TextureFormat,
        TextureSampleType, TextureViewDimension,VertexAttribute, VertexBufferLayout,
        VertexFormat, VertexStepMode
    },
    render_asset::RenderAssets,
    renderer::RenderDevice,
    RenderApp, RenderSet, texture::{Image, BevyDefault},
};
use bevy_ecs::{
    system::{lifetimeless::{Read, SRes}, SystemParamItem, SystemState},
    prelude::*,
    query::QueryItem,
};
use bevy_pbr::{
    MeshPipelineKey, MeshUniform, MeshPipeline,
    SetMeshViewBindGroup, SetMeshBindGroup,
};
use bevy_core_pipeline::core_3d::Transparent3d;
use bytemuck::{Pod, Zeroable};
use bevy_derive::Deref;
use crate::{ParticleSystem, SortParticleByDepth};

/// Plugin to render 3D billboard particles using instancing
pub struct ParticleInstancingPlugin;

impl Plugin for ParticleInstancingPlugin {
    fn build(&self, app: &mut App) {
        // A new data type `[ParticleSystemInstancedData]` will be extracted
        app.add_plugin(ExtractComponentPlugin::<ParticleSystemInstancedData>::default());
        // Adds a plane needed to render billboards particles
        app.init_resource::<BillboardMeshHandle>();
        app
            .sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawParticleSystem>()
            .init_resource::<ParticlePipeline>()
            .init_resource::<SpecializedMeshPipelines<ParticlePipeline>>()
            .add_system(queue_custom.in_set(RenderSet::Queue))
            .add_system(prepare_particle_system_draw_data.in_set(RenderSet::Prepare));
    }
}

/// The base plane for all billboard particles
#[derive(Resource)]
pub struct BillboardMeshHandle(pub Handle<Mesh>);

impl FromWorld for BillboardMeshHandle {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh_handle: Handle<Mesh> = meshes.add(Mesh::from(shape::Plane {
            size: -0.5,
            subdivisions: 0,
        }));
        BillboardMeshHandle(mesh_handle)
    }
}

/// Per instance particle data
#[allow(clippy::pedantic)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct ParticleBillboardInstanceData {
    /// Each particle position
    pub position: Vec3,
    /// Each particle scale
    pub scale: f32,
    /// Each particle velocity
    pub velocity: Vec3,
    /// Each particle rotation
    pub rotation: f32,
    /// Each particle alignment vector in world space
    pub alignment: Vec3,
    /// Each particle color
    pub color: [f32; 4],
}

/// All the instanced data from a single particle system.
/// Each particle ([`Entity`]) is associated with its instance data ([`ParticleBillboardInstanceData`])
#[derive(Component, Deref, Debug)]
pub struct ParticleSystemInstancedData(pub BTreeMap<Entity, ParticleBillboardInstanceData>);

/// Extract (Clone) the particle data from the world for rendering.
impl ExtractComponent for ParticleSystemInstancedData {
    type Query = (&'static ParticleSystemInstancedData, Option<&'static Handle<Image>>, Option<&'static SortParticleByDepth>);
    type Filter = With<ParticleSystem>;
    type Out = ExtractedInstancedData;

    fn extract_component((item, texture_handle, sort): QueryItem<'_, Self::Query>) -> Option<ExtractedInstancedData> {
        // Extract all Values from the BTreeMap and make a Vec out of them.
        // This will be useful to give a slice of the data to the buffers.
        // See crate::render::prepare_particle_system_draw_data()
        Some(ExtractedInstancedData {
            instance_data: item.0.values().copied().collect::<Vec<_>>(),
            texture: texture_handle.cloned(),
            sort_by_depth: matches!(sort, Some(_)),
        })
    }
}

/// Needed to extract the data from the [`BTreeMap`] into an array to pass to GPU for instancing
#[derive(Component, Debug)]
pub struct ExtractedInstancedData {
    /// instance_data holds all per-particle data
    pub instance_data: Vec<ParticleBillboardInstanceData>,
    /// texture holds the texture shared by all particles
    pub texture: Option<Handle<Image>>,
    /// wether or not we should sort the particles by depth before rendering
    pub sort_by_depth: bool,
}

/// Indicates that a particle must be rendered as instanced data.
/// The entity is the particle system that owns this particle.
#[derive(Debug, Component)]
pub struct InstancedParticle(pub Entity);

/// Describes the components needed to render the particle system in 3D
#[derive(Bundle)]
pub struct ParticleSystemInstancedDataBundle {
    /// The given particle mesh, can only be a plane until custom mesh particle rendering is implemented
    pub mesh_handle: Handle<Mesh>,
    /// Needed for rendering
    pub computed_visibility: ComputedVisibility,
    /// All owned particles instance data
    pub inst_data: ParticleSystemInstancedData,
}

// Queue all 3D rendered particle systems
#[allow(clippy::too_many_arguments)]
fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<ParticlePipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<ParticlePipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<(Entity, &MeshUniform, &Handle<Mesh>), With<ExtractedInstancedData>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions.read().id::<DrawParticleSystem>();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

    for (view, mut transparent_phase) in &mut views {
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();
        for (entity, mesh_uniform, mesh_handle) in &material_meshes {
            if let Some(mesh) = meshes.get(mesh_handle) {
                let key =
                    view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                let pipeline = pipelines
                    .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                    .unwrap();
                transparent_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: rangefinder.distance(&mesh_uniform.transform),
                });
            }
        }
    }
}

/// Packed particle system data, ready for rendering
#[derive(Component)]
pub struct ParticleSystemDrawData {
    /// Instance Buffer, with all instances data
    buffer: Buffer,
    /// Instance count
    length: usize,
    /// Particle system data
    ps_bind_group: BindGroup,
}

fn prepare_particle_system_draw_data(
    mut commands: Commands,
    mut particle_system_query: Query<(Entity, &mut ExtractedInstancedData)>,
    extracted_view: Query<&ExtractedView>,
    render_device: Res<RenderDevice>,
    pipeline: Res<ParticlePipeline>,
    textures: Res<RenderAssets<Image>>,
) {
    for (entity, mut extracted_instance_data) in particle_system_query.iter_mut() {

        // Sort the particles only if required by the provided settings
        if extracted_instance_data.sort_by_depth {
            // Retrieve the extracted instance data and sort it according to the first given camera (how to know its the main one ?)
            // WARNING: Here we assume that there is only one ExtractedView at this stage
            let camera_global_position = extracted_view.get_single().unwrap().transform.translation();
            
            // Create a Vec with the depth calculated
            let distances: Vec<f32> = extracted_instance_data.instance_data.iter().map(|data| {
                - data.position.distance_squared(camera_global_position)
            }).collect();

            // Create a Vec with indices
            let mut indices: Vec<usize> = (0..distances.len()).collect();

            // Sort the indices Vec according to the distances
            indices.sort_unstable_by(|&a, &b|
                distances[a].partial_cmp(&distances[b]).unwrap_or(Ordering::Less)
            );

            // We then sort the extracted_instance_data according to the indices Vec
            for i in 0..indices.len() {
                while i != indices[i] {
                    let swap_idx = indices[i];
                    extracted_instance_data.instance_data.swap(i, swap_idx);
                    indices.swap(i, swap_idx);
                }
            }

            // The standard sort method requires to calculate the square distance twice for each index.
            /*extracted_instance_data.instance_data.sort_by(|&a, &b| 
            //    b.position.distance_squared(camera_global_position)
            //        .partial_cmp(&a.position.distance_squared(camera_global_position))
            //        .unwrap_or(Ordering::Equal)
            //);*/
        }

        // Make a buffer out of the extracted instance data
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: {
                bytemuck::cast_slice(extracted_instance_data.instance_data.as_slice())
            },
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        // If no texture was provided, use the dummy texture of the mesh pipeline `[MeshPipeline::dummy_white_gpu_image]`
        let my_texture = if let Some(tex) = &extracted_instance_data.texture {
            textures.get(tex).unwrap()
        } else {
            &pipeline.mesh_pipeline.dummy_white_gpu_image
        };

        // Create the bind group for the particle system
        let ps_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("particleSystemInfo BindGroup"),
            layout: &pipeline.custom_particle_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&my_texture.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&my_texture.sampler),
                },
            ],
        });

        // Adds the prepared data to the world
        commands.entity(entity).insert(
        ParticleSystemDrawData {
            buffer,
            length: extracted_instance_data.instance_data.len(),
            ps_bind_group,
        });
    }
}

/// Describes the pipeline to render billboard particles
#[derive(Resource)]
pub struct ParticlePipeline {
    /// Shader containing the vertex and fragment functions
    shader: Handle<Shader>,
    /// The standard mesh pipeline
    mesh_pipeline: MeshPipeline,
    /// The layout to bind the particle system data
    custom_particle_layout: BindGroupLayout,
}

impl FromWorld for ParticlePipeline {
    fn from_world(world: &mut World) -> Self {
        // Get the render device...
        let mut system_state: SystemState<(
            Res<RenderDevice>,
        )> = SystemState::new(world);
        let render_device = system_state.get_mut(world).0;
        // ...And create the BindGroupLayout we will need to bind the particle system data (which is only a texture, for now)
        let bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("instance texture bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }, 
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Import the shader
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/instancing.wgsl");

        // Get the standard mesh pipeline
        let mesh_pipeline = world.resource::<MeshPipeline>();
        
        ParticlePipeline {
            shader,
            mesh_pipeline:              mesh_pipeline.clone(),
            custom_particle_layout:     bind_group_layout,
        }
    }
}

// Specialize the mesh pipeline
impl SpecializedMeshPipeline for ParticlePipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {

        // Start from the standard mesh pipeline
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        // Use the particle vertex shader
        descriptor.vertex.shader = self.shader.clone();

        // Send instances data
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<ParticleBillboardInstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                // (`ParticleBillboardInstanceData::position`, `ParticleBillboardInstanceData::scale`) as float32x4
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
                // [`ParticleBillboardInstanceData::velocity`, `ParticleBillboardInstanceData::rotation`] as float32x4
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4,
                },
                // Particle alignment with velocity as float32x4
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: VertexFormat::Float32x4.size() * 2,
                    shader_location: 5,
                },
                // `ParticleBillboardInstanceData::color` as float32x4
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: (VertexFormat::Float32x4.size() * 2) + VertexFormat::Float32x3.size(),
                    shader_location: 6,
                },
            ],
        });

        // Use the particle fragment shader
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();

        // see https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/mesh.rs
        let format = if key.contains(MeshPipelineKey::HDR) {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        // WARNING: Since particles are not sorted by depth, the alpha blending might get very weird and poppy
        // with particles that overlap each other!
        // The user should be able to set manually standard blending, premultiplied, and additive blending at least.
        //let blend = Some(BlendState::ALPHA_BLENDING);
        let blend = Some(BlendState::ALPHA_BLENDING);
        
        descriptor.fragment.as_mut().unwrap().targets = vec![Some(ColorTargetState {
            write_mask:ColorWrites::ALL,
            format,
            blend,
        })];

        // Adds the particle system data layout
        descriptor.layout.push(self.custom_particle_layout.clone());

        Ok(descriptor)
    }
}

// Describes the steps to follow to draw the particle system
type DrawParticleSystem = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawBillboardParticles,
);

/// Send all data to GPU and draw
pub struct DrawBillboardParticles;

impl<P: PhaseItem> RenderCommand<P> for DrawBillboardParticles {
    type Param = SRes<RenderAssets<Mesh>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = (Read<Handle<Mesh>>, Read<ParticleSystemDrawData>);

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        (mesh_handle, ps_data): (&'w Handle<Mesh>, &'w ParticleSystemDrawData),
        meshes: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {

        // Send mesh data (vertices)
        let Some(gpu_mesh) = meshes.into_inner().get(mesh_handle) else { return RenderCommandResult::Failure };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));

        // Send instances data
        pass.set_vertex_buffer(1, ps_data.buffer.slice(..));

        // Send particle system data
        pass.set_bind_group(
            2, // 0 and 1 are used by the view and mesh bind groups, see `[DrawParticleSystem]`
            &ps_data.ps_bind_group,
            &[]
        );

        // Draw
        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..u32::try_from(ps_data.length).unwrap());
            }
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw(0..*vertex_count, 0..u32::try_from(ps_data.length).unwrap());
            }
        }

        RenderCommandResult::Success
    }
}
