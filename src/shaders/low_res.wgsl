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
@group(0) @binding(2) var textures: texture_2d_array<f32>;

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
    let full_dim: vec2u = textureDimensions(textures);
    let aspect_ratio_corrected_uv = fragment.uv * vec2f(fragment.lower_right.xy) / vec2f(full_dim.xy);

    let texture_dims = vec2f(fragment.lower_right.xy);
    let pixel_size = fwidth(aspect_ratio_corrected_uv) * texture_dims * 0.5;
    let tx = aspect_ratio_corrected_uv * texture_dims;
    let mod_tx = (tx - 0.5) % 1.0;
    let snapped = smoothstep(0.5 - (pixel_size / 2.0), 0.5 + (pixel_size / 2.0), mod_tx);
    let correction = snapped - mod_tx;
    let corrected_uv = aspect_ratio_corrected_uv + (correction / texture_dims);

    let color: vec4f = textureSample(textures, textures_sampler, corrected_uv, fragment.lower_right.z);
    let premultiplied_color: vec4f = vec4f(color.rgb * pow(color.a, 1.0 / 2.0), color.a);
    return premultiplied_color;
}
