struct VertexIn {
    @location(0) position: vec2f,
    @location(1) color: vec3f,
};

struct VertexOut {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f,
};

@vertex
fn vertex_main(vertex: VertexIn) -> VertexOut {
    return VertexOut(
	vec4f(vertex.position, 0.0, 1.0),
	vertex.color,
    );
}

@fragment
fn fragment_main(vertex: VertexOut) -> @location(0) vec4f {
    return vec4f(vertex.color, 1.0);
}

@group(0) @binding(0) var low_res_texture: texture_2d<f32>;
@group(0) @binding(1) var low_res_sampler: sampler;

struct TTVertexIn {
    @location(0) position: vec2f,
    @location(1) uv_coord: vec2f,
};

struct TTVertexOut {
    @builtin(position) position: vec4f,
    @location(0) uv_coord: vec2f,
}

@vertex
fn texture_to_texture_vertex_main(vertex: TTVertexIn) -> TTVertexOut {
    return TTVertexOut(vec4f(vertex.position, 0.0, 1.0), vertex.uv_coord);
}

@fragment
fn texture_to_texture_fragment_main(vertex: TTVertexOut) -> @location(0) vec4f {
    return textureSample(low_res_texture, low_res_sampler, vertex.uv_coord);
}
