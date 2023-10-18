// TODO: Game struct
// TODO: Game.new
// TODO: Game.run ?
// TODO: Game.process_input
// TODO: Game.update
// TODO: Game.render
// TODO: How will I play sounds?
// TODO: Clear window with a color
// TODO: I will need to track keystate myself, possible with a set
// TODO: Simulate a lower resolution

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
