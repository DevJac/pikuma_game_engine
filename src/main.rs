// TODO: Game.run ?
// TODO: Game.process_input
// TODO: Game.update
// TODO: Game.render
// TODO: How will I play sounds?
// TODO: Clear window with a color
// TODO: I will need to track keystate myself, possible with a set
// TODO: Simulate a lower resolution
// TODO: Create a way to draw PNGs at given coordinates
// TODO: Setup a good logging system, write some logs
// TODO: Load an image and show it on the screen
// TODO: Come up with something better than unwrap-based error handling
use pollster::FutureExt as _;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct TextureVertex {
    position: glam::Vec2,
    uv: glam::Vec2,
}

const TEXTURE_VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &[
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 0,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 4 * 2,
        shader_location: 1,
    },
];

fn square() -> Vec<TextureVertex> {
    let v0 = TextureVertex {
        position: glam::Vec2::new(-1.0, 1.0),
        uv: glam::Vec2::new(0.0, 0.0),
    };
    let v1 = TextureVertex {
        position: glam::Vec2::new(-1.0, -1.0),
        uv: glam::Vec2::new(0.0, 1.0),
    };
    let v2 = TextureVertex {
        position: glam::Vec2::new(1.0, -1.0),
        uv: glam::Vec2::new(1.0, 1.0),
    };
    let v3 = TextureVertex {
        position: glam::Vec2::new(1.0, 1.0),
        uv: glam::Vec2::new(1.0, 0.0),
    };
    vec![v0, v1, v3, v3, v1, v2]
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

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Vertex {
    position: glam::Vec2,
    color: glam::Vec3,
}

const VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &[
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 0,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x3,
        offset: 4 * 2,
        shader_location: 1,
    },
];

// TODO: Make a TextureVertex struct that holds a position and uv coords
// TODO: Make a square function that returns a unit square of TextureVertex

fn triangle(angle_degrees: f32) -> Vec<Vertex> {
    let top_vert = glam::Vec2::new(0.0, 1.5);
    vec![
        Vertex {
            position: rotate_cc(0.0 + angle_degrees * 90.0) * top_vert,
            color: glam::Vec3::new(1.0, 0.0, 0.0),
        },
        Vertex {
            position: rotate_cc(120.0 + angle_degrees * 90.0) * top_vert,
            color: glam::Vec3::new(0.0, 1.0, 0.0),
        },
        Vertex {
            position: rotate_cc(240.0 + angle_degrees * 90.0) * top_vert,
            color: glam::Vec3::new(0.0, 0.0, 1.0),
        },
        Vertex {
            position: rotate_cc(0.0 + angle_degrees * 60.0) * top_vert * 0.8,
            color: glam::Vec3::new(1.0, 1.0, 1.0),
        },
        Vertex {
            position: rotate_cc(120.0 + angle_degrees * 60.0) * top_vert * 0.8,
            color: glam::Vec3::new(1.0, 1.0, 1.0),
        },
        Vertex {
            position: rotate_cc(240.0 + angle_degrees * 60.0) * top_vert * 0.8,
            color: glam::Vec3::new(1.0, 1.0, 1.0),
        },
    ]
}

struct Renderer {
    window: winit::window::Window,
    width: u32,
    height: u32,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Renderer {
    // TODO:
    fn new(window: winit::window::Window, width: u32, height: u32) -> Self {
        let instance_descriptor = wgpu::InstanceDescriptor::default();
        log::debug!("Creating default Instance: {:?}", &instance_descriptor);
        let instance: wgpu::Instance = wgpu::Instance::new(instance_descriptor);
        log::debug!("Creating Surface");
        let surface: wgpu::Surface = unsafe { instance.create_surface(&window) }.unwrap();
        let request_adapter_options = wgpu::RequestAdapterOptions::default();
        log::debug!("Creating default Adapter: {:?}", &request_adapter_options);
        let adapter: wgpu::Adapter = instance
            .request_adapter(&request_adapter_options)
            .block_on()
            .unwrap();
        let device_descriptor = wgpu::DeviceDescriptor::default();
        log::debug!("Creating default Device: {:?}", &device_descriptor);
        let (device, queue): (wgpu::Device, wgpu::Queue) = adapter
            .request_device(&device_descriptor, None)
            .block_on()
            .unwrap();
        Self {
            window,
            width,
            height,
            surface,
            device,
            queue,
        }
    }

    // TODO:
    fn draw_image() {
        // TODO: Fill image vectors with images to be drawn.
        // We need to keep a data structure in CPU memory of images we want to be drawn.
        // Perhaps with multiple layers to control which images are drawn on top of others.

        // TODO: How will we remove images from GPU memory when they are no longer used?
        // We wont. We will keep all images in memory permanantly for now.
    }

    // TODO:
    fn draw() {
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
    }
}

struct Game {
    // square_vertex_buffer: wgpu::Buffer,
    // low_res_render_pipeline: wgpu::RenderPipeline,
    // low_res_texture_view: wgpu::TextureView,
    // low_res_texture_resolved_view: wgpu::TextureView,
    // surface_render_pipeline: wgpu::RenderPipeline,
    // surface_render_bind_group: wgpu::BindGroup,
    // image: wgpu::Texture,
}

impl Game {
    fn new(window: winit::window::Window, width: u32, height: u32) -> Self {
        // TODO: Log all these things we're creating
        // TODO: Especially log the default instances so we can review their settings
        let instance_descriptor = wgpu::InstanceDescriptor::default();
        log::debug!("Creating default Instance: {:?}", &instance_descriptor);
        let instance: wgpu::Instance = wgpu::Instance::new(instance_descriptor);
        log::debug!("Creating Surface");
        let surface: wgpu::Surface = unsafe { instance.create_surface(&window) }.unwrap();
        let request_adapter_options = wgpu::RequestAdapterOptions::default();
        log::debug!("Creating default Adapter: {:?}", &request_adapter_options);
        let adapter: wgpu::Adapter = instance
            .request_adapter(&request_adapter_options)
            .block_on()
            .unwrap();
        let device_descriptor = wgpu::DeviceDescriptor::default();
        log::debug!("Creating default Device: {:?}", &device_descriptor);
        let (device, queue): (wgpu::Device, wgpu::Queue) = adapter
            .request_device(&device_descriptor, None)
            .block_on()
            .unwrap();
        // let square_verts = square();
        // let square_vert_bytes: &[u8] = bytemuck::cast_slice(square_verts.as_slice());
        // let square_vertex_buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("square buffer"),
        //     size: square_vert_bytes.len() as u64,
        //     usage: wgpu::BufferUsages::VERTEX,
        //     mapped_at_creation: true,
        // });
        // square_vertex_buffer
        //     .slice(..)
        //     .get_mapped_range_mut()
        //     .copy_from_slice(square_vert_bytes);
        // square_vertex_buffer.unmap();
        // let shader_module: wgpu::ShaderModule =
        //     device.create_shader_module(wgpu::include_wgsl!("shaders/main.wgsl"));
        // let low_res_render_pipeline =
        //     device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //         label: Some("render pipeline"),
        //         layout: None,
        //         vertex: wgpu::VertexState {
        //             module: &shader_module,
        //             entry_point: "vertex_main",
        //             buffers: &[wgpu::VertexBufferLayout {
        //                 array_stride: std::mem::size_of::<Vertex>() as u64,
        //                 step_mode: wgpu::VertexStepMode::Vertex,
        //                 attributes: VERTEX_ATTRIBUTES,
        //             }],
        //         },
        //         primitive: wgpu::PrimitiveState::default(),
        //         depth_stencil: None,
        //         multisample: wgpu::MultisampleState {
        //             count: 4,
        //             mask: !0,
        //             alpha_to_coverage_enabled: false,
        //         },
        //         fragment: Some(wgpu::FragmentState {
        //             module: &shader_module,
        //             entry_point: "fragment_main",
        //             targets: &[Some(wgpu::ColorTargetState {
        //                 format: wgpu::TextureFormat::Bgra8UnormSrgb,
        //                 blend: None,
        //                 write_mask: wgpu::ColorWrites::ALL,
        //             })],
        //         }),
        //         multiview: None,
        //     });
        // let low_res_texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
        //     label: Some("low res texture"),
        //     size: wgpu::Extent3d {
        //         width,
        //         height,
        //         depth_or_array_layers: 1,
        //     },
        //     mip_level_count: 1,
        //     sample_count: 4,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::Bgra8UnormSrgb,
        //     usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        //     view_formats: &[],
        // });
        // let low_res_texture_resolved: wgpu::Texture =
        //     device.create_texture(&wgpu::TextureDescriptor {
        //         label: Some("low res texture resolved"),
        //         size: wgpu::Extent3d {
        //             width,
        //             height,
        //             depth_or_array_layers: 1,
        //         },
        //         mip_level_count: 1,
        //         sample_count: 1,
        //         dimension: wgpu::TextureDimension::D2,
        //         format: wgpu::TextureFormat::Bgra8UnormSrgb,
        //         usage: wgpu::TextureUsages::RENDER_ATTACHMENT
        //             | wgpu::TextureUsages::TEXTURE_BINDING,
        //         view_formats: &[],
        //     });
        // let surface_render_pipeline =
        //     device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //         label: Some("surface render pipeline"),
        //         layout: None,
        //         vertex: wgpu::VertexState {
        //             module: &shader_module,
        //             entry_point: "texture_to_texture_vertex_main",
        //             buffers: &[wgpu::VertexBufferLayout {
        //                 array_stride: std::mem::size_of::<TextureVertex>() as u64,
        //                 step_mode: wgpu::VertexStepMode::Vertex,
        //                 attributes: TEXTURE_VERTEX_ATTRIBUTES,
        //             }],
        //         },
        //         primitive: wgpu::PrimitiveState::default(),
        //         depth_stencil: None,
        //         multisample: wgpu::MultisampleState::default(),
        //         fragment: Some(wgpu::FragmentState {
        //             module: &shader_module,
        //             entry_point: "texture_to_texture_fragment_main",
        //             targets: &[Some(wgpu::ColorTargetState {
        //                 format: wgpu::TextureFormat::Bgra8UnormSrgb,
        //                 blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        //                 write_mask: wgpu::ColorWrites::ALL,
        //             })],
        //         }),
        //         multiview: None,
        //     });
        // let surface_render_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        //     label: Some("low res sampler"),
        //     address_mode_u: wgpu::AddressMode::ClampToEdge,
        //     address_mode_v: wgpu::AddressMode::ClampToEdge,
        //     address_mode_w: wgpu::AddressMode::ClampToEdge,
        //     mag_filter: wgpu::FilterMode::Nearest,
        //     min_filter: wgpu::FilterMode::Nearest,
        //     mipmap_filter: wgpu::FilterMode::Nearest,
        //     lod_min_clamp: 0.0,
        //     lod_max_clamp: 0.0,
        //     compare: None,
        //     anisotropy_clamp: 1,
        //     border_color: None,
        // });
        // let low_res_texture_view =
        //     low_res_texture.create_view(&wgpu::TextureViewDescriptor::default());
        // let low_res_texture_resolved_view =
        //     low_res_texture_resolved.create_view(&wgpu::TextureViewDescriptor::default());
        // let surface_render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("surface render bind group"),
        //     layout: &surface_render_pipeline.get_bind_group_layout(0),
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&low_res_texture_resolved_view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&surface_render_sampler),
        //         },
        //     ],
        // });
        // let dynamic_image: image::DynamicImage = image::io::Reader::open("assets/images/tree.png")
        //     .unwrap()
        //     .decode()
        //     .unwrap();
        // let image = device.create_texture(&wgpu::TextureDescriptor {
        //     label: Some("image"),
        //     size: wgpu::Extent3d {
        //         width: dynamic_image.width(),
        //         height: dynamic_image.height(),
        //         depth_or_array_layers: 1,
        //     },
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::Rgba8UnormSrgb,
        //     usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        //     view_formats: &[],
        // });
        // queue.write_texture(
        //     wgpu::ImageCopyTexture {
        //         texture: &image,
        //         mip_level: 0,
        //         origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
        //         aspect: wgpu::TextureAspect::All,
        //     },
        //     dynamic_image.as_bytes(),
        //     wgpu::ImageDataLayout {
        //         offset: 0,
        //         bytes_per_row: Some(dynamic_image.width() * 4),
        //         rows_per_image: Some(dynamic_image.height()),
        //     },
        //     wgpu::Extent3d {
        //         width: dynamic_image.width(),
        //         height: dynamic_image.height(),
        //         depth_or_array_layers: 1,
        //     },
        // );
        let game = Game {
            window,
            width,
            height,
            surface,
            device,
            queue,
        };
        // game.configure_surface();
        game
    }

    fn configure_surface(&self) {
        let window_inner_size = self.window.inner_size();
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: window_inner_size.width,
                height: window_inner_size.height,
                present_mode: wgpu::PresentMode::AutoNoVsync,
                // The window surface does not support alpha
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            },
        );
    }

    fn render(&self, t: std::time::Duration) {
        // TODO: Log all these things we're creating
        // TODO: Especially log the default instances so we can review their settings
        let triangle_vertices = triangle(t.as_secs_f32());
        let triangle_vertice_bytes: &[u8] = bytemuck::cast_slice(triangle_vertices.as_slice());
        let vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex buffer"),
            size: triangle_vertice_bytes.len() as u64,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: true,
        });
        vertex_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(triangle_vertice_bytes);
        vertex_buffer.unmap();
        let mut command_encoder: wgpu::CommandEncoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let surface_texture: wgpu::SurfaceTexture = self.surface.get_current_texture().unwrap();
        let surface_texture_view: wgpu::TextureView = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut low_res_render_pass: wgpu::RenderPass =
                command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("low res render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.low_res_texture_view,
                        resolve_target: Some(&self.low_res_texture_resolved_view),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                // We're rendering to a window surface which ignores alpha
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            low_res_render_pass.set_pipeline(&self.low_res_render_pipeline);
            low_res_render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            low_res_render_pass.draw(0..6, 0..1);
        }
        {
            let affine_transform = glam::Affine2::from_cols_array(&[0.5, 0.0, 0.0, 0.5, -0.5, 0.5]);
            let square_verts: Vec<TextureVertex> = square()
                .iter()
                .map(|tv| TextureVertex {
                    position: affine_transform.transform_point2(tv.position),
                    uv: tv.uv,
                })
                .collect();
            let square_vert_bytes: &[u8] = bytemuck::cast_slice(square_verts.as_slice());
            let square_vertex_buffer: wgpu::Buffer =
                self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("square buffer"),
                    size: square_vert_bytes.len() as u64,
                    usage: wgpu::BufferUsages::VERTEX,
                    mapped_at_creation: true,
                });
            square_vertex_buffer
                .slice(..)
                .get_mapped_range_mut()
                .copy_from_slice(square_vert_bytes);
            square_vertex_buffer.unmap();
            let image_view = self
                .image
                .create_view(&wgpu::TextureViewDescriptor::default());
            let image_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("image sampler"),
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
            let image_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("image bind group"),
                layout: &self.surface_render_pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&image_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&image_sampler),
                    },
                ],
            });
            let mut image_render_pass: wgpu::RenderPass =
                command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("image render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.low_res_texture_resolved_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            image_render_pass.set_pipeline(&self.surface_render_pipeline);
            image_render_pass.set_bind_group(0, &image_bind_group, &[]);
            image_render_pass.set_vertex_buffer(0, square_vertex_buffer.slice(..));
            image_render_pass.draw(0..6, 0..1);
        }
        {
            let mut surface_render_pass: wgpu::RenderPass =
                command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("surface render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &surface_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                // We're rendering to a window surface which ignores alpha
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            surface_render_pass.set_pipeline(&self.surface_render_pipeline);
            surface_render_pass.set_bind_group(0, &self.surface_render_bind_group, &[]);
            surface_render_pass.set_vertex_buffer(0, self.square_vertex_buffer.slice(..));
            surface_render_pass.draw(0..6, 0..1);
        }
        self.queue.submit([command_encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
}

struct FPSStats {
    /// The half life (in seconds) of samples
    half_life: f32,
    /// mean
    mean: f32,
    /// variance
    variance: f32,
}

impl FPSStats {
    fn new(half_life: f32) -> Self {
        Self {
            half_life,
            mean: 1.0 / 60.0,
            variance: 0.0,
        }
    }

    fn update(&mut self, frame_time: f32) {
        let alpha: f32 = 2.0_f32.powf(-frame_time / self.half_life);
        self.mean = alpha * self.mean + (1.0 - alpha) * frame_time;
        self.variance = alpha * self.variance + (1.0 - alpha) * (self.mean - frame_time).powi(2);
    }

    fn mean(&self) -> f32 {
        self.mean
    }

    fn variance(&self) -> f32 {
        self.variance
    }

    /// Standard deviation
    fn std(&self) -> f32 {
        self.variance.sqrt()
    }
}

fn main() {
    // TODO: Process input
    // TODO: Update game state
    // TODO: Render
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window: winit::window::Window = winit::window::Window::new(&event_loop).unwrap();
    let game = Game::new(window, 80, 60);
    let start_time = std::time::Instant::now();
    let mut last_render_time = start_time;
    let mut last_fps_log_time = start_time;
    let mut render_time_stats = FPSStats::new(1.0);
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop
        .run(move |event, event_loop_window_target| {
            let time_since_start: std::time::Duration = std::time::Instant::now() - start_time;
            match event {
                winit::event::Event::WindowEvent {
                    window_id: _,
                    event: window_event,
                } => match window_event {
                    winit::event::WindowEvent::CloseRequested => {
                        event_loop_window_target.exit();
                    }
                    winit::event::WindowEvent::KeyboardInput {
                        device_id: _,
                        event:
                            winit::event::KeyEvent {
                                physical_key: _,
                                logical_key:
                                    winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                                text: _,
                                location: _,
                                state: _,
                                repeat: _,
                                ..
                            },
                        is_synthetic: _,
                    } => {
                        event_loop_window_target.exit();
                    }
                    winit::event::WindowEvent::Resized(_) => {
                        game.configure_surface();
                    }
                    _ => {}
                },
                winit::event::Event::DeviceEvent {
                    device_id: _,
                    event: _device_event,
                } => {
                    // TODO: Handle button presses
                    // TODO: Track button states
                }
                winit::event::Event::AboutToWait => {
                    game.render(time_since_start);
                    let now = std::time::Instant::now();
                    let render_time_seconds: f32 = (now - last_render_time).as_secs_f32();
                    render_time_stats.update(render_time_seconds);
                    last_render_time = now;
                    if now - last_fps_log_time > std::time::Duration::from_secs(10) {
                        last_fps_log_time = now;
                        let fps = 1.0 / render_time_stats.mean();
                        let fps_std = render_time_stats.std() / render_time_stats.mean().powi(2);
                        log::info!("FPS: {:.0} ± {:.0}", fps, fps_std);
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
