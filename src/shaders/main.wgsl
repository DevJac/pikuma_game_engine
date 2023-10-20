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
