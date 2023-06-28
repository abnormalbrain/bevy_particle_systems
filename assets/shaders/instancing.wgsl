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
    @location(4) i_velocity_rotation: vec4<f32>,
    @location(5) i_alignment: vec3<f32>,
    @location(6) i_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    // instance world position
    let instance_position = vertex.i_pos_scale.xyz;
    // instance world scale
    let instance_scale = vertex.i_pos_scale.w;
    // instance rotation (in radians)
    let instance_rotation = vertex.i_velocity_rotation.w;
    // instance velocity
    let instance_velocity = vertex.i_velocity_rotation.xyz;
    // instance alignment, vector that needs to face the velocity.
    // is (0,0,0) if no alignment required
    let instance_alignment = vertex.i_alignment;

    // resolve the rotation implied by the alignment
    // PERFORMANCE NOTE: This branch could be more efficient with a defined #if, implemented through the pipeline key
    var cumulated_rotation = instance_rotation;
    if (length(instance_alignment) > 0.5) {
        // clip alignment
        let v1 = instance_alignment.xy;
        // clip velocity
        let v2 = normalize((view.view_proj * vec4<f32>(instance_velocity, 0.0)).xy);

        // equals cos(angle_between_vectors) since they are normalized
        let dot = dot(v1, v2);
        let cross = v1.x * v2.y - v1.y * v2.x;
        let angle = atan2(cross, dot);

        cumulated_rotation += angle;
    }

    // get the UVs with rotation applied
    let rot_sin = sin(cumulated_rotation);
    let rot_cos = cos(cumulated_rotation);
    let centered_uvs = vec2<f32>(
        vertex.uv.x - 0.5,
        vertex.uv.y - 0.5
    );
    let rotated_uvs = vec2<f32>(
        centered_uvs.x * rot_cos - centered_uvs.y * rot_sin,
        centered_uvs.x * rot_sin + centered_uvs.y * rot_cos,
    );

    // Get the up and right direction in world space from the projection matrix
    // and normalize them to get a unit vector in world space
    // For better performance, this could be done on CPU side
    let abs_right = normalize((view.inverse_view_proj * vec4<f32>(1.0, 0.0, 0.0, 0.0)).xyz);
    let abs_up = normalize((view.inverse_view_proj * vec4<f32>(0.0, 1.0, 0.0, 0.0)).xyz);

    // resolve billboard position using billboard up and right vector and plane uv
    let w_pos:    vec3<f32> =
        instance_position
        + (abs_right.xyz *  rotated_uvs.x *   instance_scale)
        + (abs_up.xyz *     rotated_uvs.y *   instance_scale);

    var out: VertexOutput;
    out.clip_position = mesh_position_world_to_clip(vec4<f32>(w_pos, 1.0));
    out.uv = vertex.uv;
    out.color = vertex.i_color;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(instance_texture, instance_sampler, in.uv.xy);
    return color * in.color;
}