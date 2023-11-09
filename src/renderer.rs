use pollster::FutureExt as _;
use wgpu::util::DeviceExt as _;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Vertex {
    position: glam::Vec2,
    uv: glam::Vec2,
}

const VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &[
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2, // size = 4 * 2 = 8
        offset: 0,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2, // size = 4 * 2 = 8
        offset: 8,
        shader_location: 1,
    },
];

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct TextureVertex {
    position: glam::Vec2,
    uv: glam::Vec2,
    lower_right: glam::UVec3,
}

const TEXTURE_VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &[
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2, // size = 4 * 2 = 8
        offset: 0,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2, // size = 4 * 2 = 8
        offset: 8,
        shader_location: 1,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Uint32x3, // size = 4 * 3 = 12
        offset: 16,
        shader_location: 2,
    },
];

/// Normalized device coordinates (NDC)
fn ndc_square() -> Vec<Vertex> {
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
    vec![v0, v1, v2, v2, v3, v0]
}

fn square(
    position: glam::UVec2,
    texture_size: glam::UVec2,
    texture_index: u32,
) -> Vec<TextureVertex> {
    let lower_right = glam::UVec3::new(texture_size.x, texture_size.y, texture_index);
    let v0 = TextureVertex {
        position: glam::Vec2::new(position.x as f32, position.y as f32),
        uv: glam::Vec2::new(0.0, 0.0),
        lower_right,
    };
    let v1 = TextureVertex {
        position: glam::Vec2::new(position.x as f32, (position.y + texture_size.y) as f32),
        uv: glam::Vec2::new(0.0, 1.0),
        lower_right,
    };
    let v2 = TextureVertex {
        position: glam::Vec2::new(
            (position.x + texture_size.x) as f32,
            (position.y + texture_size.y) as f32,
        ),
        uv: glam::Vec2::new(1.0, 1.0),
        lower_right,
    };
    let v3 = TextureVertex {
        position: glam::Vec2::new((position.x + texture_size.x) as f32, position.y as f32),
        uv: glam::Vec2::new(1.0, 0.0),
        lower_right,
    };
    vec![v0, v1, v2, v2, v3, v0]
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
pub enum TankOrTree {
    Tank,
    Tree,
}

struct LowResPass {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl LowResPass {
    fn new(
        device: &wgpu::Device,
        canvas_width: u32,
        canvas_height: u32,
        preferred_format: wgpu::TextureFormat,
        textures_view: &wgpu::TextureView,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("LowResPass.texture"),
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
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/low_res.wgsl"));
        let pipeline: wgpu::RenderPipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("LowResPass.pipeline"),
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
            label: Some("LowResPass.uniform_buffer"),
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
        let sampler: wgpu::Sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("LowResPass.bind_group"),
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
                    resource: wgpu::BindingResource::TextureView(&textures_view),
                },
            ],
        });
        Self {
            texture,
            texture_view,
            pipeline,
            bind_group,
        }
    }
}

pub struct Renderer {
    // WGPU Stuff
    surface: wgpu::Surface,
    preferred_format: wgpu::TextureFormat,
    device: wgpu::Device,
    queue: wgpu::Queue,
    // Render Passes
    low_res_pass: LowResPass,
    // Textures / sprites
    textures: wgpu::Texture,
    textures_view: wgpu::TextureView,
    vertex_buffer: wgpu::Buffer,
    vertex_buffer_write_offset: u64,
    surface_bind_group: wgpu::BindGroup,
    surface_vertex_buffer: wgpu::Buffer,
    surface_render_pipeline: wgpu::RenderPipeline,
    surface_aspect_ratio_uniform: wgpu::Buffer,
    // TODO: Use an instance buffer as well
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
        let (textures, textures_view) = Renderer::load_textures(&device, &queue);
        let low_res_pass = LowResPass::new(
            &device,
            canvas_width,
            canvas_height,
            preferred_format,
            &textures_view,
        );
        let vertex_buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Renderer::new vertex_buffer"),
            size: 1000,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let surface_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/surface_render.wgsl"));
        let surface_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Renderer::new surface_render_pipeline"),
                layout: None,
                vertex: wgpu::VertexState {
                    module: &surface_shader,
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
                    module: &surface_shader,
                    entry_point: "fragment_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: preferred_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });
        let surface_aspect_ratio_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Renderer::new aspect_ratio_uniform"),
            size: std::mem::size_of::<glam::Vec2>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let surface_sampler: wgpu::Sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Renderer::new low_res_sampler"),
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
        let surface_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Renderer::new surface_bind_group"),
            layout: &surface_render_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&surface_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&low_res_pass.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &surface_aspect_ratio_uniform,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });
        let surface_square = ndc_square();
        let surface_square_bytes: &[u8] = bytemuck::cast_slice(surface_square.as_slice());
        let surface_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Renderer::new surface_vertex_buffer"),
            contents: surface_square_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self {
            window,
            surface,
            preferred_format,
            device,
            queue,
            low_res_pass,
            textures,
            textures_view,
            vertex_buffer,
            vertex_buffer_write_offset: 0,
            surface_bind_group,
            surface_vertex_buffer,
            surface_render_pipeline,
            surface_aspect_ratio_uniform,
        }
    }

    fn load_textures(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let textures: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Renderer::new textures"),
            size: wgpu::Extent3d {
                width: 32,
                height: 32,
                // TODO: Texture layers needs to be dynamic or something. Hard code 2 for now.
                depth_or_array_layers: 2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let textures_view = textures.create_view(&wgpu::TextureViewDescriptor::default());
        let tank: image::DynamicImage =
            image::io::Reader::open("assets/images/tank-panther-right.png")
                .unwrap()
                .decode()
                .unwrap();
        let tree: image::DynamicImage = image::io::Reader::open("assets/images/tree.png")
            .unwrap()
            .decode()
            .unwrap();
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &textures,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            tank.as_bytes(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(tank.width() * 4),
                rows_per_image: Some(tank.height()),
            },
            wgpu::Extent3d {
                width: tank.width(),
                height: tank.height(),
                depth_or_array_layers: 1,
            },
        );
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &textures,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 1 },
                aspect: wgpu::TextureAspect::All,
            },
            tree.as_bytes(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(tree.width() * 4),
                rows_per_image: Some(tree.height()),
            },
            wgpu::Extent3d {
                width: tree.width(),
                height: tree.height(),
                depth_or_array_layers: 1,
            },
        );
        (textures, textures_view)
    }

    pub fn configure_surface(&self) {
        let window_inner_size = self.window.inner_size();
        let canvas_to_surface_ratio_width: f32 =
            (self.low_res_pass.texture.width() as f32) / (window_inner_size.width as f32);
        let canvas_to_surface_ratio_height: f32 =
            (self.low_res_pass.texture.height() as f32) / (window_inner_size.height as f32);
        let maximum_canvas_to_surface_ratio: f32 =
            canvas_to_surface_ratio_width.max(canvas_to_surface_ratio_height);
        let canvas_scale = glam::Vec2::new(
            canvas_to_surface_ratio_width / maximum_canvas_to_surface_ratio,
            canvas_to_surface_ratio_height / maximum_canvas_to_surface_ratio,
        );
        self.queue.write_buffer(
            &self.surface_aspect_ratio_uniform,
            0,
            bytemuck::bytes_of(&canvas_scale),
        );
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

    pub fn draw_image(&mut self, tank_or_tree: TankOrTree, location: glam::UVec2) {
        let texture_index = match tank_or_tree {
            TankOrTree::Tank => 0,
            TankOrTree::Tree => 1,
        };
        let texture_vertex_size = std::mem::size_of::<TextureVertex>() as u64;
        let square_vertices = square(location, glam::UVec2::new(32, 32), texture_index);
        let square_bytes: &[u8] = bytemuck::cast_slice(square_vertices.as_slice());
        let start = self.vertex_buffer_write_offset;
        let end = start + square_vertices.len() as u64;
        self.queue.write_buffer(
            &self.vertex_buffer,
            start * texture_vertex_size,
            square_bytes,
        );
        self.vertex_buffer_write_offset = end;
    }

    pub fn draw(&mut self) {
        // Steps:
        // - Unmap vertex buffer
        // - (We might have to copy from the write buffer to a non-mappable buffer?)
        // - Render vertex buffer
        // - Map vertex buffer and wait for mapping to finish

        // TODO: Setup vertex buffer
        // We need to know the size of our vertex buffer before creating.
        // We will need to create a CPU data structure (vec based) to store the images
        // we want to draw, and then move them into a vertex buffer when it's time to render.

        // TODO: Low res render pass
        // Low res render pass will draw many images onto the low res texture.
        // A vertex buffer will be setup (prior to this point)
        // containing quads, uvs, and image indexes that should be drawn.
        // A render pipeline will be created earlier (maybe a RenderBundle?).

        // TODO: Surface render pass
        // Position a quad and draw the low res texture to the surface, then present the surface.
        // Keep aspect ratio of low res texture.
        let surface_texture: wgpu::SurfaceTexture = self.surface.get_current_texture().unwrap();
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut command_encoder: wgpu::CommandEncoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Renderer::draw command_encoder"),
                });
        {
            let mut low_res_render_pass: wgpu::RenderPass =
                command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Renderer::draw low_res_render_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.low_res_pass.texture_view,
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
            low_res_render_pass.set_pipeline(&self.low_res_pass.pipeline);
            low_res_render_pass.set_bind_group(0, &self.low_res_pass.bind_group, &[]);
            low_res_render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            low_res_render_pass.draw(0..self.vertex_buffer_write_offset as u32, 0..1);
        }
        {
            self.vertex_buffer_write_offset = 0;
            let mut surface_render_pass =
                command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Renderer::draw surface_render_pass"),
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
            surface_render_pass.set_pipeline(&self.surface_render_pipeline);
            surface_render_pass.set_bind_group(0, &self.surface_bind_group, &[]);
            surface_render_pass.set_vertex_buffer(0, self.surface_vertex_buffer.slice(..));
            surface_render_pass.draw(0..6, 0..1);
        }
        self.queue.submit([command_encoder.finish()]);
        surface_texture.present();
    }
}
