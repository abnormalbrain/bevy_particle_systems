// Custom billboard shader

// Made from bevy_pbr/src/render/mesh.wgsl
// https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/mesh.wgsl
// Many commented inputs have been kept to facilitate further pbr features implementation for this billboard shader

#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

@group(1) @binding(5)
var<uniform> billboard_size: vec2<f32>;
@group(1) @binding(4)
var<uniform> color: vec4<f32>;
@group(1) @binding(2)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(3)
var base_color_sampler: sampler;

// needs to be imported after bindings
#import bevy_pbr::mesh_functions


// Vertex input data
struct Vertex {
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
//#ifdef VERTEX_NORMALS
//    @location(1) normal: vec3<f32>,
//#endif
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
//#ifdef VERTEX_TANGENTS
//    @location(3) tangent: vec4<f32>,
//#endif
//#ifdef VERTEX_COLORS
//    @location(4) color: vec4<f32>,
//#endif
//#ifdef SKINNED
//    @location(5) joint_indices: vec4<u32>,
//    @location(6) joint_weights: vec4<f32>,
//#endif
};


// 
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #ifdef VERTEX_POSITIONS
        @location(0) world_position: vec4<f32>,
    #endif
    #ifdef VERTEX_UVS
        @location(2) uv: vec2<f32>,
    #endif
};



@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef SKINNED
    var model = skin_model(vertex.joint_indices, vertex.joint_weights);
#else
    var model = mesh.model;
#endif

// Calculate billboard positions
// retrieve camera position from the view uniform buffer
let cam_pos:    vec3<f32> = view.world_position;
// use the inverse view projection matrix to get the camera up vector
let cam_up:     vec3<f32> = view.inverse_view_proj[1].xyz;
// get the pivot of the plane in world position
let instance_position = mesh_position_local_to_world(model, vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
// get the vector mesh pivot -> camera
let view:   vec3<f32> = normalize(cam_pos - instance_position);
// cross with camera up to get the billboard right vector
let right:  vec3<f32> = normalize(cross(cam_up, view));
// cross billboard right with view to get billboard up vector
let up:     vec3<f32> = cross(view, right);
// resolve billboard position using billboard up and right vector and plane uv
let w_pos:    vec3<f32> =
    instance_position
    + (right *  (vertex.uv.x - 0.5) *   billboard_size.x)
    + (up *     (vertex.uv.y - 0.5) *   billboard_size.y).xyz;

//#ifdef VERTEX_NORMALS
//    out.world_normal = view;
//#endif

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_POSITIONS
    let world_pos = vec4<f32>(w_pos, 1.0);
    // we use our custom billboard position as vertex world position
    out.world_position = world_pos;
    out.clip_position = mesh_position_world_to_clip(out.world_position);
#endif

//#ifdef VERTEX_TANGENTS
//    out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
//#endif

//#ifdef VERTEX_COLORS
//    out.color = vertex.color;
//#endif

    return out;
}

// It would be better to have a custom pipeline that sends camera's up and right unit vectors
// instead of recalculate them per vertex
fn get_billboard_world_position(vertex_uv: vec2<f32>, model: mat4x4<f32>) -> vec4<f32> {
    let inv_view_proj = view.inverse_view_proj;
    let cam_pos:    vec3<f32> = view.world_position;
    let cam_up:     vec3<f32> = inv_view_proj[1].xyz;
    let instance_position = mesh_position_local_to_world(model, vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;

    let view:   vec3<f32> = normalize(cam_pos - instance_position);
    let right:  vec3<f32> = normalize(cross(cam_up, view));
    let up:     vec3<f32> = cross(view, right);
    let w_pos:    vec3<f32> =
        instance_position
        + (right *  (vertex_uv.x - 0.5) *   billboard_size.x)
        + (up *     (vertex_uv.y - 0.5) *   billboard_size.y).xyz;
    
    return vec4<f32>(w_pos, 1.0);
}

// Billboard fragment shader
@fragment
fn fragment(
    #ifdef VERTEX_POSITIONS
        @builtin(position) clip_position: vec4<f32>,
        @location(0) world_position: vec4<f32>,
    #endif
    #ifdef VERTEX_UVS
        @location(2) uv: vec2<f32>,
    #endif
) -> @location(0) vec4<f32> {
    return color * textureSample(base_color_texture, base_color_sampler, uv);
}
