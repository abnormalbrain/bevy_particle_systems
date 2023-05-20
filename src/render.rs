//! Defines Data and methods used for rendering the particles.

use std::collections::BTreeMap;
use bevy_asset::{Handle, AssetServer, Assets, Asset, AddAsset};
use bevy_math::{Vec2, Vec3};
use bevy_app::{App, Plugin, StartupSet};
use bevy_render::{
    prelude::{Msaa, SpatialBundle, shape,},
    extract_component::{ ExtractComponentPlugin, ExtractComponent},
    mesh::{GpuBufferInfo, MeshVertexBufferLayout},
    render_phase::{
        AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand,
        RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass,
    },
    view::{ExtractedView, NoFrustumCulling, visibility::ComputedVisibility},
    texture::GpuImage,
    render_resource::*,
    render_asset::RenderAssets,
    renderer::RenderDevice,
    RenderApp, RenderSet, texture::Image, prelude::Color, prelude::Mesh,
};
use bevy_ecs::{
    system::{lifetimeless::*, SystemParamItem, SystemState},
    prelude::*,
    query::QueryItem,
};
use bevy_pbr::{
    Material, AlphaMode, MeshPipelineKey, MeshUniform, MeshPipeline,
    SetMeshViewBindGroup, SetMeshBindGroup,
};
use bevy_reflect::TypeUuid;
use bevy_core_pipeline::core_3d::Transparent3d;
use bytemuck::{Pod, Zeroable};
use bevy_derive::Deref;

pub struct ParticleInstancingPlugin;

impl Plugin for ParticleInstancingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<ParticleSystemInstancedData>::default());
        app.add_startup_system(setup_billboard_resource.in_base_set(StartupSet::PreStartup));
        app
            .sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<ParticlePipeline>()
            .init_resource::<SpecializedMeshPipelines<ParticlePipeline>>()
            .init_resource::<TestTexture>()
            //.init_resource::<BillboardMeshHandle>()
            .add_system(queue_custom.in_set(RenderSet::Queue))
            .add_system(prepare_instance_buffers.in_set(RenderSet::Prepare));
    }
}

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

#[derive(Resource)]
pub struct TestTexture(pub Handle<Image>);

impl FromWorld for TestTexture {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture_handle: Handle<Image> = asset_server.load("gabe-idle-run.png");
        TestTexture(texture_handle)
    }
}

#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct ParticleBillboardInstanceData {
    pub position: Vec3,
    pub scale: f32,
    pub rotation: [f32; 4],
    pub color: [f32; 4],
}

/// All the instanced data from a single particle system.
/// Each particle (Entity) is associated with its instance data (ParticleBillboardInstanceData)
#[derive(Component, Deref, Debug)]
pub struct ParticleSystemInstancedData(pub BTreeMap<Entity, ParticleBillboardInstanceData>);

/// Needed to extract the data from the BTreeMap into an array to pass to GPU for instancing
#[derive(Component, Debug)]
pub struct ExtractedInstancedData(pub Vec<ParticleBillboardInstanceData>);

/// Clone the particle data from the world for rendering.
impl ExtractComponent for ParticleSystemInstancedData {
    type Query = &'static ParticleSystemInstancedData;
    type Filter = ();
    type Out = ExtractedInstancedData;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<ExtractedInstancedData> {
        // Extract all Values from the BTreeMap and make a Vec out of them.
        // This will be useful to give a slice of the data to the buffers.
        // See `[crate::render::prepare_instance_buffers()]`
        Some(ExtractedInstancedData(item.0.iter().map(|(_, v)| *v).collect::<Vec<_>>()))
    }
}

#[derive(Debug, Component)]
/// Indicates that a particle must be rendered as instanced data.
/// The entity is the particle system that owns this instanced data.
pub struct InstancedParticle(pub Entity);

#[derive(Bundle)]
pub struct ParticleSystemInstancedDataBundle {
    pub mesh_handle: Handle<Mesh>,
    pub computed_visibility: ComputedVisibility,
    pub inst_data: ParticleSystemInstancedData,
    pub disabled_frustrum_culling: NoFrustumCulling,
}

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
    let draw_custom = transparent_3d_draw_functions.read().id::<DrawCustom>();

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

#[derive(Component)]
pub struct ParticleInstanceBuffer {
    buffer: Buffer,
    length: usize,
    ps_bind_group: BindGroup,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &ExtractedInstancedData, Option<&Handle<Image>>)>,
    render_device: Res<RenderDevice>,
    pipeline: Res<ParticlePipeline>,
    test_texture: Res<TestTexture>,
    textures: Res<RenderAssets<Image>>,
) {
    for (entity, instance_data, texture) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: {
                bytemuck::cast_slice(instance_data.0.as_slice())
            },
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let my_texture = if let Some(tex) = texture {
            textures.get(&tex).unwrap()
        } else {
            textures.get(&test_texture.0).unwrap()
        };

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

        commands.entity(entity).insert(
        ParticleInstanceBuffer {
            buffer,
            length: instance_data.0.len(),
            ps_bind_group,
        });
    }
}

#[derive(Resource)]
pub struct ParticlePipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    custom_particle_layout: BindGroupLayout,
}

impl FromWorld for ParticlePipeline {
    fn from_world(world: &mut World) -> Self {
        // added
        let mut system_state: SystemState<(
            Res<RenderDevice>,
        )> = SystemState::new(world);
        let render_device = system_state.get_mut(world).0;

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
        // end added

        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/instancing.wgsl");

        let mesh_pipeline = world.resource::<MeshPipeline>();
        
        ParticlePipeline {
            shader:                     shader,
            mesh_pipeline:              mesh_pipeline.clone(),
            custom_particle_layout:     bind_group_layout,
        }
    }
}

impl SpecializedMeshPipeline for ParticlePipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<ParticleBillboardInstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                // [`ParticleBillboardInstanceData::position`, `ParticleBillboardInstanceData::scale`] as float32x4
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
                // `ParticleBillboardInstanceData::rotation` as float32x4
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4,
                },
                // `ParticleBillboardInstanceData::color` as float32x4
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size()*2,
                    shader_location: 5,
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();

        // Adds our uniform data layout
        descriptor.layout.push(self.custom_particle_layout.clone());

        Ok(descriptor)
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawMeshInstanced,
);


pub struct DrawMeshInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = SRes<RenderAssets<Mesh>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = (Read<Handle<Mesh>>, Read<ParticleInstanceBuffer>);

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        (mesh_handle, instance_buffer): (&'w Handle<Mesh>, &'w ParticleInstanceBuffer),
        meshes: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));
        pass.set_bind_group(2, &instance_buffer.ps_bind_group, &[]);

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw(0..*vertex_count, 0..instance_buffer.length as u32);
            }
        }

        RenderCommandResult::Success
    }
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