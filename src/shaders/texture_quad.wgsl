struct TextureVertex {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
    @location(2) lower_right: vec3u,
};

struct TextureFragment {
    @builtin(position) position: vec4f,
    @location(1) uv: vec2f,
    @location(1) @interpolate(flat) lower_right: vec3u,
};

@group(0) @binding(0) var sampler: sampler;
@group(0) @binding(1) var textures: texture_2d_array<f32>;

@vertex
fn vertex_main(vertex: TextureVertex) -> TextureFragment {
    return TextureFragment(vertex.position, vertex.uv, vertex.lower_right);
}

@fragment fragment_main(fragment: TextureFragment) -> @location(0) vec4f {
    // All textures in the texture_2d_array must be the same size.
    // However, some textures might only be partially initialized,
    // because our game assets are not all the same size.
    // We need to adjust the UV coordinates so that (1, 1) refers to
    // the lower right of the initialized portion of the texture.
    let full_dim = textureDimensions(textures);
    let adjusted_uv = vec2f(
	fragment.uv.x * (f32(fragment.lower_right.x) / full_dim.x),
	fragment.uv.y * (f32(fragment.lower_right.y) / full_dim.y)
    );
    return textureSample(textures, sampler, adjusted_uv, fragment.lower_right.z);
}
