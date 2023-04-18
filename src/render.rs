//! Defines Data and methods used for rendering the particles.

use std::collections::BTreeMap;
use bevy_asset::{Handle, AssetServer};
use bevy_math::{Vec2, Vec3};
use bevy_app::{App, Plugin};
use bevy_render::{
    prelude::{Msaa, SpatialBundle},
    extract_component::{ ExtractComponentPlugin, ExtractComponent},
    mesh::{GpuBufferInfo, MeshVertexBufferLayout},
    render_phase::{
        AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand,
        RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass,
    },
    view::{ExtractedView, NoFrustumCulling},
    render_resource::*,
    render_asset::RenderAssets,
    renderer::RenderDevice,
    RenderApp, RenderSet, texture::Image, prelude::Color, prelude::Mesh,
};
use bevy_ecs::{
    system::{lifetimeless::*, SystemParamItem},
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
        app
            .sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<ParticlePipeline>()
            .init_resource::<SpecializedMeshPipelines<ParticlePipeline>>()
            .add_system(queue_custom.in_set(RenderSet::Queue))
            .add_system(prepare_instance_buffers.in_set(RenderSet::Prepare));
    }
}

/// Defines a billboard material for particles
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690cdae-d528-45bb-8225-97e2a4f056d0"]
pub struct BillboardMaterial {
    #[uniform(4)]
    pub color: Color,
    #[uniform(5)]
    pub size: Vec2,
    #[texture(2)]
    #[sampler(3)]
    pub texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
} 
 
/// Make BillboardMaterial a Material, and assign the billboard shaders to it
impl Material for BillboardMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/particle_billboard.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/particle_billboard.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

#[derive(Resource)]
pub struct BillboardMeshHandle(pub Handle<Mesh>);

#[derive(Resource)]
pub struct BillboardAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<BillboardMaterial>,
}

#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct ParticleBillboardInstanceData {
    pub position: Vec3,
    pub scale: f32,
    pub color: [f32; 4],
}

/// All the instanced data from a single particle system.
/// Each particle (Entity) is associated with its instance data (ParticleBillboardInstanceData)
#[derive(Component, Deref, Debug)]
pub struct ParticleSystemInstancedData(pub BTreeMap<Entity, ParticleBillboardInstanceData>);

/// Needed to extract the data from the BTreeMap into an array to pass to GPU for instancing
#[derive(Component, Debug)]
pub struct ExtractedInstancedData(pub Vec<ParticleBillboardInstanceData>);
/// Clone the data from the world for rendering.
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
    pub spacial: SpatialBundle,
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
pub struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &ExtractedInstancedData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: {
                //let values_slice = instance_data.0.clone();
                bytemuck::cast_slice(instance_data.0.as_slice())
            },
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.0.len(),
        });
    }
}

#[derive(Resource)]
pub struct ParticlePipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
}

impl FromWorld for ParticlePipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/instancing.wgsl");

        let mesh_pipeline = world.resource::<MeshPipeline>();

        ParticlePipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
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
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4,
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
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
    type ItemWorldQuery = (Read<Handle<Mesh>>, Read<InstanceBuffer>);

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        (mesh_handle, instance_buffer): (&'w Handle<Mesh>, &'w InstanceBuffer),
        meshes: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

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
