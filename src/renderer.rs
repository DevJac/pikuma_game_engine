use pollster::FutureExt as _;
use wgpu::util::DeviceExt as _;

#[derive(Clone, Copy)]
pub struct SpriteIndex(u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sprite {
    file: std::path::PathBuf,
    top_left: glam::UVec2,
    width_height: glam::UVec2,
}

impl Sprite {
    pub fn new(file: std::path::PathBuf, top_left: glam::UVec2, width_height: glam::UVec2) -> Self {
        Self {
            file,
            top_left,
            width_height,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Vertex {
    position: glam::Vec2,
    uv: glam::Vec2,
}

const VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &[
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2, // position size = 4 * 2 = 8
        offset: 0,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2, // uv size = 4 * 2 = 8
        offset: 8,
        shader_location: 1,
    },
];

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct TextureVertex {
    position: glam::Vec3,
    uv: glam::Vec2,
    lower_right: glam::UVec3,
}

const TEXTURE_VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &[
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x3, // position size = 4 * 3 = 12
        offset: 0,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2, // uv size = 4 * 2 = 8
        offset: 12,
        shader_location: 1,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Uint32x3, // lower_right size = 4 * 3 = 12
        offset: 20,
        shader_location: 2,
    },
];

const SQUARE_VERTS: u32 = 6;

/// Normalized device coordinates (NDC)
fn ndc_square() -> [Vertex; SQUARE_VERTS as usize] {
    let v0 = Vertex {
        position: glam::Vec2::new(-1.0, -1.0),
        uv: glam::Vec2::new(0.0, 0.0),
    };
    let v1 = Vertex {
        position: glam::Vec2::new(-1.0, 1.0),
        uv: glam::Vec2::new(0.0, 1.0),
    };
    let v2 = Vertex {
        position: glam::Vec2::new(1.0, 1.0),
        uv: glam::Vec2::new(1.0, 1.0),
    };
    let v3 = Vertex {
        position: glam::Vec2::new(1.0, -1.0),
        uv: glam::Vec2::new(1.0, 0.0),
    };
    [v0, v1, v2, v2, v3, v0]
}

fn square(
    position: glam::UVec2,
    z: f32,
    texture_size: glam::UVec2,
    texture_index: u32,
) -> [TextureVertex; SQUARE_VERTS as usize] {
    let lower_right = glam::UVec3::new(texture_size.x, texture_size.y, texture_index);
    let v0 = TextureVertex {
        position: glam::Vec3::new(position.x as f32, position.y as f32, z),
        uv: glam::Vec2::new(0.0, 0.0),
        lower_right,
    };
    let v1 = TextureVertex {
        position: glam::Vec3::new(position.x as f32, (position.y + texture_size.y) as f32, z),
        uv: glam::Vec2::new(0.0, 1.0),
        lower_right,
    };
    let v2 = TextureVertex {
        position: glam::Vec3::new(
            (position.x + texture_size.x) as f32,
            (position.y + texture_size.y) as f32,
            z,
        ),
        uv: glam::Vec2::new(1.0, 1.0),
        lower_right,
    };
    let v3 = TextureVertex {
        position: glam::Vec3::new((position.x + texture_size.x) as f32, position.y as f32, z),
        uv: glam::Vec2::new(1.0, 0.0),
        lower_right,
    };
    [v0, v1, v2, v2, v3, v0]
}

/// Counter-clockwise rotation matrix
fn rotate_cc(angle_degrees: f32) -> glam::Mat2 {
    let angle_radians = angle_degrees.to_radians();
    glam::Mat2::from_cols_array(&[
        angle_radians.cos(),
        angle_radians.sin(),
        -angle_radians.sin(),
        angle_radians.cos(),
    ])
}

// TODO: We need a better resource handling strategy
#[derive(Clone, Copy)]
pub enum TankOrTree {
    Tank,
    Tree,
}

struct LowResPass {
    low_res_texture: wgpu::Texture,
    low_res_texture_view: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer_cpu: Vec<u8>,
    vertex_buffer: wgpu::Buffer,
    vertex_buffer_vert_count: u32,
    // Sprites
    sprites: wgpu::Texture,
    loaded_sprites: Vec<Sprite>,
}

impl LowResPass {
    fn new(
        device: &wgpu::Device,
        canvas_width: u32,
        canvas_height: u32,
        preferred_format: wgpu::TextureFormat,
    ) -> Self {
        let low_res_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("low res texture"),
            size: wgpu::Extent3d {
                width: canvas_width,
                height: canvas_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: preferred_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let low_res_texture_view =
            low_res_texture.create_view(&wgpu::TextureViewDescriptor::default());
        // TODO: Stop including the shader in the compiled binary. Compile them at runtime.
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/low_res.wgsl"));
        let pipeline: wgpu::RenderPipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("low res pipeline"),
                layout: None,
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vertex_main",
                    // TODO: We should use instance buffers for repeated values
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<TextureVertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: TEXTURE_VERTEX_ATTRIBUTES,
                    }],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fragment_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: preferred_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });
        let uniform_buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("low res uniform buffer"),
            size: std::mem::size_of::<glam::UVec2>() as u64,
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: true,
        });
        uniform_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::bytes_of(&glam::UVec2::new(
                canvas_width,
                canvas_height,
            )));
        uniform_buffer.unmap();
        let sampler: wgpu::Sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("low res sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        let sprites: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("low res sprites"),
            size: wgpu::Extent3d {
                width: 32,
                height: 32,
                depth_or_array_layers: 256,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let sprites_view: wgpu::TextureView =
            sprites.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("low res bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&sprites_view),
                },
            ],
        });
        // TODO: Use an instance buffer as well
        // TODO: What should we do about this hard-coded static buffer size?
        let vertex_buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("low res vertex buffer"),
            size: 100_000,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            low_res_texture,
            low_res_texture_view,
            pipeline,
            bind_group,
            vertex_buffer_cpu: Vec::new(),
            vertex_buffer,
            vertex_buffer_vert_count: 0,
            sprites,
            loaded_sprites: Vec::new(),
        }
    }

    fn load_sprite(&mut self, queue: &wgpu::Queue, sprite: Sprite) -> SpriteIndex {
        if let Some(existing_index) = self
            .loaded_sprites
            .iter()
            .position(|loaded_sprite| *loaded_sprite == sprite)
        {
            return SpriteIndex(existing_index as u32);
        }
        let sprite_image: image::RgbaImage = image::io::Reader::open(&sprite.file)
            .unwrap_or_else(|_| panic!("couldn't open sprite file ({:?})", &sprite.file))
            .decode()
            .unwrap_or_else(|_| panic!("couldn't decode sprite file ({:?})", &sprite.file))
            .crop(
                sprite.top_left.x,
                sprite.top_left.y,
                sprite.width_height.x,
                sprite.width_height.y,
            )
            .into_rgba8();
        let sprite_index = self.loaded_sprites.len() as u32;
        let bytes_per_pixel = 4;
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.sprites,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: sprite_index,
                },
                aspect: wgpu::TextureAspect::All,
            },
            sprite_image.as_raw(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(sprite_image.width() * bytes_per_pixel),
                rows_per_image: Some(sprite_image.height()),
            },
            wgpu::Extent3d {
                width: sprite_image.width(),
                height: sprite_image.height(),
                depth_or_array_layers: 1,
            },
        );
        self.loaded_sprites.push(sprite);
        log::debug!("Loaded new sprite at index: {}", sprite_index);
        SpriteIndex(sprite_index)
    }

    fn draw_image(&mut self, sprite_index: SpriteIndex, sprite_z: f32, location: glam::UVec2) {
        let sprite_width_height: glam::UVec2 =
            self.loaded_sprites[sprite_index.0 as usize].width_height;
        let square_vertices = square(location, sprite_z, sprite_width_height, sprite_index.0);
        let square_bytes: &[u8] = bytemuck::cast_slice(square_vertices.as_slice());
        self.vertex_buffer_cpu.extend_from_slice(square_bytes);
        self.vertex_buffer_vert_count += 1;
    }

    fn draw(&mut self, queue: &wgpu::Queue, command_encoder: &mut wgpu::CommandEncoder) {
        queue.write_buffer(&self.vertex_buffer, 0, self.vertex_buffer_cpu.as_slice());
        let mut pass: wgpu::RenderPass =
            command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("low res render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.low_res_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.15,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.vertex_buffer_vert_count * SQUARE_VERTS, 0..1);
        self.vertex_buffer_cpu.clear();
        self.vertex_buffer_vert_count = 0;
    }
}

struct SurfacePass {
    pipeline: wgpu::RenderPipeline,
    aspect_ratio_uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
}

impl SurfacePass {
    fn new(
        device: &wgpu::Device,
        preferred_format: wgpu::TextureFormat,
        low_res_texture_view: &wgpu::TextureView,
    ) -> Self {
        // TODO: Stop including the shader in the compiled binary. Compile them at runtime.
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/surface.wgsl"));
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("surface pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: VERTEX_ATTRIBUTES,
                }],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: preferred_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });
        let aspect_ratio_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("surface uniform"),
            size: std::mem::size_of::<glam::Vec2>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler: wgpu::Sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("surface sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("surface bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &aspect_ratio_uniform,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&low_res_texture_view),
                },
            ],
        });
        let ndc_square = ndc_square();
        let ndc_square_bytes: &[u8] = bytemuck::cast_slice(ndc_square.as_slice());
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("surface vertex buffer"),
            contents: ndc_square_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self {
            pipeline,
            aspect_ratio_uniform,
            bind_group,
            vertex_buffer,
        }
    }

    fn update_aspect_ratio(&self, queue: &wgpu::Queue, scales: glam::Vec2) {
        queue.write_buffer(&self.aspect_ratio_uniform, 0, bytemuck::bytes_of(&scales));
    }

    fn draw(&self, command_encoder: &mut wgpu::CommandEncoder, surface_view: &wgpu::TextureView) {
        let mut surface_render_pass =
            command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("surface render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        surface_render_pass.set_pipeline(&self.pipeline);
        surface_render_pass.set_bind_group(0, &self.bind_group, &[]);
        surface_render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        surface_render_pass.draw(0..SQUARE_VERTS, 0..1);
    }
}

pub struct Renderer {
    // WGPU stuff
    surface: wgpu::Surface,
    preferred_format: wgpu::TextureFormat,
    device: wgpu::Device,
    queue: wgpu::Queue,
    // Render passes
    low_res_pass: LowResPass,
    surface_pass: SurfacePass,
    // Window
    // unsafe: window must live longer than surface.
    window: winit::window::Window,
}

impl Renderer {
    pub fn new(window: winit::window::Window, canvas_width: u32, canvas_height: u32) -> Self {
        let instance: wgpu::Instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        // unsafe: The window must live longer than its surface.
        let surface: wgpu::Surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter: wgpu::Adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .block_on()
            .unwrap();
        let preferred_format: wgpu::TextureFormat =
            *surface.get_capabilities(&adapter).formats.get(0).unwrap();
        log::debug!("Preferred format is: {:?}", &preferred_format);
        let (device, queue): (wgpu::Device, wgpu::Queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on()
            .unwrap();
        log::debug!("WGPU setup");
        let low_res_pass = LowResPass::new(&device, canvas_width, canvas_height, preferred_format);
        let surface_pass = SurfacePass::new(
            &device,
            preferred_format,
            &low_res_pass.low_res_texture_view,
        );
        Self {
            window,
            surface,
            preferred_format,
            device,
            queue,
            low_res_pass,
            surface_pass,
        }
    }

    pub fn configure_surface(&self) {
        let window_inner_size = self.window.inner_size();
        let canvas_to_surface_ratio_width: f32 =
            (self.low_res_pass.low_res_texture.width() as f32) / (window_inner_size.width as f32);
        let canvas_to_surface_ratio_height: f32 =
            (self.low_res_pass.low_res_texture.height() as f32) / (window_inner_size.height as f32);
        let maximum_canvas_to_surface_ratio: f32 =
            canvas_to_surface_ratio_width.max(canvas_to_surface_ratio_height);
        let canvas_scales = glam::Vec2::new(
            canvas_to_surface_ratio_width / maximum_canvas_to_surface_ratio,
            canvas_to_surface_ratio_height / maximum_canvas_to_surface_ratio,
        );
        self.surface_pass
            .update_aspect_ratio(&self.queue, canvas_scales);
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.preferred_format,
                width: window_inner_size.width,
                height: window_inner_size.height,
                present_mode: wgpu::PresentMode::AutoNoVsync,
                // The window surface does not support alpha
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            },
        );
    }

    pub fn load_sprite(&mut self, sprite: Sprite) -> SpriteIndex {
        self.low_res_pass.load_sprite(&self.queue, sprite)
    }

    pub fn draw_image(&mut self, sprite_index: SpriteIndex, sprite_z: f32, location: glam::UVec2) {
        self.low_res_pass
            .draw_image(sprite_index, sprite_z, location);
    }

    pub fn draw(&mut self) {
        let surface_texture: wgpu::SurfaceTexture = self.surface.get_current_texture().unwrap();
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut command_encoder: wgpu::CommandEncoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("command encoder"),
                });
        self.low_res_pass.draw(&self.queue, &mut command_encoder);
        self.surface_pass.draw(&mut command_encoder, &surface_view);
        self.queue.submit([command_encoder.finish()]);
        surface_texture.present();
    }
}
