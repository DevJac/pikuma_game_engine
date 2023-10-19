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
        Game {
            window,
            surface,
            device,
            queue,
        }
    }
}

fn main() {
    // TODO: Process input
    // TODO: Update game state
    // TODO: Render
    let event_loop = winit::event_loop::EventLoop::new();
    let _window = winit::window::Window::new(&event_loop);
    event_loop.run(|event, _, control_flow| match event {
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

            // TODO: Game rendering
        }
        _ => {}
    });
}
