// TODO: Game.run ?
// TODO: Game.process_input
// TODO: Game.update
// TODO: Game.render
// TODO: How will I play sounds?
// TODO: Clear window with a color
// TODO: I will need to track keystate myself, possible with a set
// TODO: Simulate a lower resolution
use pollster::FutureExt as _;

struct Game {
    window: winit::window::Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Game {
    fn new(window: winit::window::Window) -> Self {
        // TODO: Log all these things we're creating
        // TODO: Especially log the default instances so we can review their settings
        let instance: wgpu::Instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface: wgpu::Surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter: wgpu::Adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .block_on()
            .unwrap();
        let (device, queue): (wgpu::Device, wgpu::Queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on()
            .unwrap();
        let game = Game {
            window,
            surface,
            device,
            queue,
        };
        game.configure_surface();
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
                present_mode: wgpu::PresentMode::AutoVsync,
                // The window surface does not support alpha
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![wgpu::TextureFormat::Bgra8UnormSrgb],
            },
        );
    }

    fn render(&self) {
        // TODO: Log all these things we're creating
        // TODO: Especially log the default instances so we can review their settings
        let mut command_encoder: wgpu::CommandEncoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let surface_texture: wgpu::SurfaceTexture = self.surface.get_current_texture().unwrap();
        let surface_texture_view: wgpu::TextureView = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        {
            let _render_pass: wgpu::RenderPass =
                command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("render_pass"),
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
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
        }
        self.queue.submit([command_encoder.finish()]);
        surface_texture.present();
    }
}

fn main() {
    // TODO: Process input
    // TODO: Update game state
    // TODO: Render
    let event_loop = winit::event_loop::EventLoop::new();
    let window: winit::window::Window = winit::window::Window::new(&event_loop).unwrap();
    let game = Game::new(window);
    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::WindowEvent {
            window_id: _,
            event: window_event,
        } => match window_event {
            winit::event::WindowEvent::CloseRequested => {
                control_flow.set_exit();
            }
            winit::event::WindowEvent::KeyboardInput {
                device_id: _,
                input:
                    winit::event::KeyboardInput {
                        scancode: _,
                        state: _,
                        virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                        ..
                    },
                is_synthetic: _,
            } => {
                control_flow.set_exit();
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
        winit::event::Event::MainEventsCleared => {
            // The winit docs say:
            // Programs that draw graphics continuously, like most games,
            // can render here unconditionally for simplicity.
            // See: https://docs.rs/winit/latest/winit/event/enum.Event.html#variant.MainEventsCleared
            game.render();
        }
        _ => {}
    });
}
