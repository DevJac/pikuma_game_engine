struct TextureSize {
    @location(0) width: u32,
    @location(1) height: u32,
};

struct TextureVertex {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
    @location(2) lower_right: vec3u,
};

struct TextureFragment {
    @builtin(position) position: vec4f,
    @location(1) uv: vec2f,
    @location(2) @interpolate(flat) lower_right: vec3u,
};

@group(0) @binding(0) var<uniform> texture_size: TextureSize;
@group(0) @binding(1) var textures_sampler: sampler;
@group(0) @binding(2) var textures: binding_array<texture_2d<f32>>;

@vertex
fn vertex_main(vertex: TextureVertex) -> TextureFragment {
    let ndc = vec4f(
        vertex.position.x / f32(texture_size.width) * 2.0 - 1.0,
        vertex.position.y / f32(texture_size.height) * 2.0 - 1.0,
        0.0,
        1.0,
    );
    return TextureFragment(ndc, vertex.uv, vertex.lower_right);
}

@fragment
fn fragment_main(fragment: TextureFragment) -> @location(0) vec4f {
    // All textures in the texture_2d_array must be the same size.
    // However, some textures might only be partially initialized,
    // because our game assets are not all the same size.
    // We need to adjust the UV coordinates so that (1, 1) refers to
    // the lower right of the initialized portion of the texture.
    let texture_index = fragment.lower_right.z;
    return textureSample(textures[texture_index], textures_sampler, fragment.uv);
}
