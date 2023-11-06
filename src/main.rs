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
use pikuma_game_engine::fps_stats::FPSStats;
use pikuma_game_engine::renderer;

struct Game {
    renderer: renderer::Renderer,
    width: u32,
    tank_location: glam::Vec2,
}

impl Game {
    fn new(window: winit::window::Window, width: u32, height: u32) -> Self {
        let renderer = renderer::Renderer::new(window, width, height);
        renderer.configure_surface();
        Game {
            renderer,
            width,
            tank_location: glam::Vec2::new(0.0, 25.0),
        }
    }

    fn configure_surface(&self) {
        self.renderer.configure_surface();
    }

    fn render(&mut self, delta_t: f32) {
        self.tank_location += glam::Vec2::new(1.0, 0.0) * delta_t;
        if self.tank_location.x > self.width as f32 {
            self.tank_location = glam::Vec2::new(0.0, 25.0);
        }
        self.renderer
            .draw_image(renderer::TankOrTree::Tree, glam::UVec2::new(20, 10));
        self.renderer.draw_image(
            renderer::TankOrTree::Tank,
            self.tank_location.round().as_uvec2(),
        );
        self.renderer.draw();
    }
}

fn main() {
    // TODO: Process input
    // TODO: Update game state
    // TODO: Render
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window: winit::window::Window = winit::window::Window::new(&event_loop).unwrap();
    let mut game = Game::new(window, 800, 600);
    let start_time = std::time::Instant::now();
    let mut last_render_time = start_time;
    let mut frame_render_seconds: f32 = 0.0;
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
                    game.render(frame_render_seconds);
                    let now = std::time::Instant::now();
                    frame_render_seconds = (now - last_render_time).as_secs_f32();
                    render_time_stats.update(frame_render_seconds);
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
