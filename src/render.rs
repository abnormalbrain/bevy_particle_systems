//! Defines Data and methods used for rendering the particles.

use bevy_asset::Handle;
use bevy_math::Vec2;
use bevy_render::{
    texture::Image, prelude::Color, prelude::Mesh
};
use bevy_ecs::prelude::Resource;
use bevy_pbr::{Material, AlphaMode};
use bevy_reflect::TypeUuid;
use bevy_render::render_resource::{AsBindGroup, ShaderRef};

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

/// Defines the plane mesh that will be used for the billboard particles
//#[derive(TypeUuid)]
//#[uuid = "f690cdae-d528-45bb-8915-97e2a4f053d0"]
//pub struct ParticleBillboardPlane(pub Mesh);

#[derive(Resource)]
pub struct BillboardAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<BillboardMaterial>,
}