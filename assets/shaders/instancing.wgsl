#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings

@group(1) @binding(0)
var<uniform> mesh: Mesh;

@group(2) @binding(0)
var instance_texture: texture_2d<f32>;

@group(2) @binding(1)
var instance_sampler: sampler;

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos_scale: vec4<f32>,
    @location(4) i_rotation: vec4<f32>,
    @location(5) i_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    // Calculate billboard positions
    // retrieve camera position from the view uniform buffer
    let cam_pos: vec3<f32> = view.world_position;
    // use the inverse view projection matrix to get the camera up vector
    let cam_up: vec3<f32> = view.inverse_view_proj[1].xyz;
    // get the pivot of the plane in world position, we don't use the mesh model matrix because
    //let instance_position = mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.i_pos_scale.xyz, 1.0)).xyz;
    let instance_position = vertex.i_pos_scale.xyz;
    // get the vector mesh pivot -> camera
    let view: vec3<f32> = normalize(cam_pos - instance_position);
    // cross with camera up to get the billboard right vector
    let right: vec3<f32> = normalize(cross(cam_up, view));
    // cross billboard right with view to get billboard up vector
    let up: vec3<f32> = cross(view, right);
    // rotate UVs to apply the rotation
    let rot_sin = sin(vertex.i_rotation.x);
    let rot_cos = cos(vertex.i_rotation.x);
    let rotated_uvs = vec2<f32>(
        vertex.uv.x * rot_cos - vertex.uv.y * rot_sin,
        vertex.uv.x * rot_sin + vertex.uv.y * rot_cos,
    );
    // resolve billboard position using billboard up and right vector and plane uv
    let w_pos:    vec3<f32> =
        instance_position
        + (right *  (rotated_uvs.x - 0.5) *   vertex.i_pos_scale.w)
        + (up *     (rotated_uvs.y - 0.5) *   vertex.i_pos_scale.w);
    //let position = vertex.position * vertex.i_pos_scale.w + vertex.i_pos_scale.xyz;
    var out: VertexOutput;
    out.clip_position = mesh_position_world_to_clip(vec4<f32>(w_pos, 1.0));
    //out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(vertex.position, 1.0));
    out.uv = vertex.uv;
    out.color = vertex.i_color;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(instance_texture, instance_sampler, in.uv.xy);
    return color * in.color;
    //return in.color;
}