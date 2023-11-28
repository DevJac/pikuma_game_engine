/// The size of the full low res canvas; e.g., 800 x 600.
struct TextureSize {
    @location(0) width: u32,
    @location(1) height: u32,
};

struct TextureVertex {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
    @location(2) texture_index: u32,
};

struct TextureFragment {
    @builtin(position) position: vec4f,
    @location(1) uv: vec2f,
    @location(2) @interpolate(flat) texture_index: u32,
};

@group(0) @binding(0) var<uniform> texture_size: TextureSize;
@group(0) @binding(1) var textures_sampler: sampler;
@group(0) @binding(2) var textures: binding_array<texture_2d<f32>>;

@vertex
fn vertex_main(vertex: TextureVertex) -> TextureFragment {
    // Adjust coordinates in our world space (e.g., somewhere in the 800 x 600 grid)
    // to normalized device coordinates (NDC, e.g., somewhere in the -1 to 1 range).
    let ndc = vec4f(
        vertex.position.x / f32(texture_size.width) * 2.0 - 1.0,
        vertex.position.y / f32(texture_size.height) * 2.0 - 1.0,
        0.0,
        1.0,
    );
    return TextureFragment(ndc, vertex.uv, vertex.texture_index);
}

@fragment
fn fragment_main(fragment: TextureFragment) -> @location(0) vec4f {
    return textureSample(textures[fragment.texture_index], textures_sampler, fragment.uv);
}
