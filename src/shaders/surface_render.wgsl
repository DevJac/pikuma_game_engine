struct Vertex {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
};

struct Fragment {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
}

@group(0) @binding(0) var low_res_sampler: sampler;
@group(0) @binding(1) var low_res_texture: texture_2d<f32>;
@group(0) @binding(2) var<uniform> aspect_ratio_scale: vec2f;

@vertex
fn vertex_main(vertex: Vertex) -> Fragment {
    return Fragment(
        vec4f(
            vertex.position.x * aspect_ratio_scale.x,
            vertex.position.y * aspect_ratio_scale.y,
            0.0,
            1.0),
        vertex.uv);
}

@fragment
fn fragment_main(vertex: Fragment) -> @location(0) vec4f {
    return textureSample(low_res_texture, low_res_sampler, vertex.uv);
}
