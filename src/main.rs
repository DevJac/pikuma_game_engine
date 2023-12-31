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
use pikuma_game_engine::renderer::Sprite;
use pikuma_game_engine::{components_systems, ecs, renderer};
use std::cell::RefCell;
use std::io::BufRead as _;
use std::rc::Rc;

struct Game {
    renderer: renderer::Renderer,
    registry: ecs::Registry,
    pressed_keys: std::collections::HashSet<winit::keyboard::PhysicalKey>,
}

impl Game {
    fn new(window: winit::window::Window, width: u32, height: u32) -> Self {
        let mut registry = ecs::Registry::new();
        let mut renderer = renderer::Renderer::new(window, width, height);
        renderer.configure_surface();

        let tree = registry.create_entity();
        let tank_1 = registry.create_entity();
        let tank_2 = registry.create_entity();
        let chopper = registry.create_entity();
        registry
            .add_component(
                tree,
                components_systems::RigidBodyComponent {
                    position: glam::Vec2::new(20.0, 10.0),
                    velocity: glam::Vec2::new(0.0, 0.0),
                },
            )
            .unwrap();
        registry
            .add_component(
                tree,
                components_systems::SpriteComponent {
                    sprite_index: renderer.load_sprite(Sprite::new(
                        "assets/images/tree.png".into(),
                        glam::UVec2::new(0, 0),
                        glam::UVec2::new(16, 32),
                    )),
                    sprite_layer: components_systems::Layer::Ground,
                    size: glam::Vec2::new(16.0, 32.0),
                },
            )
            .unwrap();
        registry
            .add_component(
                tank_1,
                components_systems::RigidBodyComponent {
                    position: glam::Vec2::new(0.0, 50.0),
                    velocity: glam::Vec2::new(10.0, 4.0),
                },
            )
            .unwrap();
        registry
            .add_component(
                tank_1,
                components_systems::SpriteComponent {
                    sprite_index: renderer.load_sprite(Sprite::new(
                        "assets/images/tank-panther-right.png".into(),
                        glam::UVec2::new(0, 0),
                        glam::UVec2::new(32, 32),
                    )),
                    sprite_layer: components_systems::Layer::Ground,
                    size: glam::Vec2::new(32.0, 32.0),
                },
            )
            .unwrap();
        registry
            .add_component(
                tank_1,
                components_systems::CollisionComponent {
                    offset: glam::Vec2::new(6.0, 6.0),
                    width_height: glam::Vec2::new(20.0, 20.0),
                },
            )
            .unwrap();
        registry
            .add_component(
                tank_2,
                components_systems::RigidBodyComponent {
                    position: glam::Vec2::new(0.0, 100.0),
                    velocity: glam::Vec2::new(10.0, 8.0),
                },
            )
            .unwrap();
        registry
            .add_component(
                tank_2,
                components_systems::SpriteComponent {
                    sprite_index: renderer.load_sprite(Sprite::new(
                        "assets/images/tank-panther-right.png".into(),
                        glam::UVec2::new(0, 0),
                        glam::UVec2::new(32, 32),
                    )),
                    sprite_layer: components_systems::Layer::Ground,
                    size: glam::Vec2::new(32.0, 32.0),
                },
            )
            .unwrap();
        registry
            .add_component(
                tank_2,
                components_systems::CollisionComponent {
                    offset: glam::Vec2::new(6.0, 6.0),
                    width_height: glam::Vec2::new(20.0, 20.0),
                },
            )
            .unwrap();
        registry
            .add_component(
                chopper,
                components_systems::RigidBodyComponent {
                    position: glam::Vec2::new(0.0, 200.0),
                    velocity: glam::Vec2::new(10.0, -3.0),
                },
            )
            .unwrap();
        registry
            .add_component(
                chopper,
                components_systems::SpriteComponent {
                    sprite_index: renderer.load_sprite(Sprite::new(
                        "assets/images/chopper-spritesheet.png".into(),
                        glam::UVec2::new(0, 0),
                        glam::UVec2::new(32, 32),
                    )),
                    sprite_layer: components_systems::Layer::Air,
                    size: glam::Vec2::new(32.0, 32.0),
                },
            )
            .unwrap();
        registry
            .add_component(
                chopper,
                components_systems::MotionAnimationComponent::new(
                    1.0 / 15.0,
                    vec![
                        renderer.load_sprite(Sprite::new(
                            "assets/images/chopper-spritesheet.png".into(),
                            glam::UVec2::new(32 * 0, 32 * 3),
                            glam::UVec2::new(32, 32),
                        )),
                        renderer.load_sprite(Sprite::new(
                            "assets/images/chopper-spritesheet.png".into(),
                            glam::UVec2::new(32 * 1, 32 * 3),
                            glam::UVec2::new(32, 32),
                        )),
                    ],
                    vec![
                        renderer.load_sprite(Sprite::new(
                            "assets/images/chopper-spritesheet.png".into(),
                            glam::UVec2::new(32 * 0, 32 * 2),
                            glam::UVec2::new(32, 32),
                        )),
                        renderer.load_sprite(Sprite::new(
                            "assets/images/chopper-spritesheet.png".into(),
                            glam::UVec2::new(32 * 1, 32 * 2),
                            glam::UVec2::new(32, 32),
                        )),
                    ],
                    vec![
                        renderer.load_sprite(Sprite::new(
                            "assets/images/chopper-spritesheet.png".into(),
                            glam::UVec2::new(32 * 0, 32 * 1),
                            glam::UVec2::new(32, 32),
                        )),
                        renderer.load_sprite(Sprite::new(
                            "assets/images/chopper-spritesheet.png".into(),
                            glam::UVec2::new(32 * 1, 32 * 1),
                            glam::UVec2::new(32, 32),
                        )),
                    ],
                    vec![
                        renderer.load_sprite(Sprite::new(
                            "assets/images/chopper-spritesheet.png".into(),
                            glam::UVec2::new(32 * 0, 32 * 0),
                            glam::UVec2::new(32, 32),
                        )),
                        renderer.load_sprite(Sprite::new(
                            "assets/images/chopper-spritesheet.png".into(),
                            glam::UVec2::new(32 * 1, 32 * 0),
                            glam::UVec2::new(32, 32),
                        )),
                    ],
                ),
            )
            .unwrap();
        registry
            .add_component(
                chopper,
                components_systems::CollisionComponent {
                    offset: glam::Vec2::new(6.0, 6.0),
                    width_height: glam::Vec2::new(20.0, 20.0),
                },
            )
            .unwrap();
        registry
            .add_component(chopper, components_systems::KeyboardControlComponent {})
            .unwrap();
        registry
            .add_component(
                chopper,
                components_systems::CameraFocusComponent {
                    focus_offset: glam::Vec2::new(16.0, 16.0),
                    viewport_size: glam::Vec2::new(800.0, 600.0),
                    map_top_left: glam::Vec2::ZERO,
                    map_bottom_right: glam::Vec2::new(25.0 * 32.0 * 2.0, 20.0 * 32.0 * 2.0),
                },
            )
            .unwrap();
        registry.add_system(Rc::new(RefCell::new(
            components_systems::MovementSystem::new(),
        )));
        registry.add_system(Rc::new(RefCell::new(
            components_systems::AnimationSystem::new(),
        )));
        registry.add_system(Rc::new(RefCell::new(
            components_systems::MotionAnimationSystem::new(),
        )));
        registry.add_system(Rc::new(RefCell::new(
            components_systems::CameraFocusSystem::new(),
        )));
        registry.add_system(Rc::new(RefCell::new(
            components_systems::RenderSystem::new(),
        )));
        registry.add_system(Rc::new(RefCell::new(
            components_systems::KeyboardControlSystem::new(),
        )));
        let collision_system = Rc::new(RefCell::new(components_systems::CollisionSystem::new()));
        registry.add_handler::<components_systems::CollisionEvent, _>(Rc::clone(&collision_system));
        registry.add_handler::<winit::keyboard::PhysicalKey, _>(Rc::clone(&collision_system));
        registry.add_system(collision_system);

        let mut game = Game {
            renderer,
            registry,
            pressed_keys: std::collections::HashSet::new(),
        };
        game.load_map("assets/tilemaps/jungle.map");
        game
    }

    /// Read tilemap and create entities for each background tile.
    fn load_map<P: AsRef<std::path::Path>>(&mut self, map_file: P) {
        let map_file = std::fs::File::open(&map_file)
            .unwrap_or_else(|_| panic!("can't read map file ({:?})", map_file.as_ref()));
        let reader = std::io::BufReader::new(map_file);
        for (row, line) in reader.lines().enumerate() {
            let line = line.expect("can't read map file line");
            for (col, tile) in line.split(',').enumerate() {
                let tile = tile.trim().parse::<u32>().expect("can't parse tile index");
                let sprite = Sprite::new(
                    "assets/tilemaps/jungle.png".into(),
                    glam::UVec2::new(32 * (tile % 10), 32 * (tile / 10)),
                    glam::UVec2::new(32, 32),
                );
                let background_tile = self.registry.create_entity();
                let map_scale = 2.0;
                self.registry
                    .add_component(
                        background_tile,
                        components_systems::RigidBodyComponent {
                            position: glam::Vec2::new(
                                32.0 * map_scale * col as f32,
                                32.0 * map_scale * row as f32,
                            ),
                            velocity: glam::Vec2::new(0.0, 0.0),
                        },
                    )
                    .unwrap();
                self.registry
                    .add_component(
                        background_tile,
                        components_systems::SpriteComponent {
                            sprite_index: self.renderer.load_sprite(sprite),
                            sprite_layer: components_systems::Layer::Background,
                            size: glam::Vec2::new(32.0 * map_scale, 32.0 * map_scale),
                        },
                    )
                    .unwrap();
            }
        }
    }

    fn configure_surface(&self) {
        self.renderer.configure_surface();
    }

    fn render(&mut self, delta_t: f32) {
        self.registry
            .run_system::<components_systems::KeyboardControlSystem>(&self.pressed_keys)
            .unwrap();
        self.registry
            .run_system::<components_systems::MovementSystem>(delta_t)
            .unwrap();
        self.registry
            .run_system::<components_systems::CollisionSystem>(&mut self.renderer)
            .unwrap();
        self.registry
            .run_system::<components_systems::AnimationSystem>(delta_t)
            .unwrap();
        self.registry
            .run_system::<components_systems::MotionAnimationSystem>(delta_t)
            .unwrap();
        self.registry
            .run_system::<components_systems::CameraFocusSystem>(&mut self.renderer)
            .unwrap();
        self.registry
            .run_system::<components_systems::RenderSystem>(&mut self.renderer)
            .unwrap();
        self.renderer.draw();
    }

    fn key_event(&mut self, key_event: winit::event::RawKeyEvent) {
        match key_event.state {
            winit::event::ElementState::Pressed => {
                let new_keypress = self.pressed_keys.insert(key_event.physical_key);
                if new_keypress {
                    self.registry.dispatch_event(key_event.physical_key);
                }
            }
            winit::event::ElementState::Released => {
                self.pressed_keys.remove(&key_event.physical_key);
            }
        }
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
        .run(move |event, event_loop_window_target| match event {
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
                winit::event::WindowEvent::KeyboardInput {
                    device_id: _,
                    event:
                        winit::event::KeyEvent {
                            physical_key,
                            logical_key: _,
                            text: _,
                            location: _,
                            state,
                            repeat: _,
                            ..
                        },
                    is_synthetic: _,
                } => {
                    game.key_event(winit::event::RawKeyEvent {
                        physical_key,
                        state,
                    });
                }
                winit::event::WindowEvent::Resized(_) => {
                    game.configure_surface();
                }
                _ => {}
            },
            winit::event::Event::DeviceEvent {
                device_id: _,
                event: device_event,
            } => match device_event {
                winit::event::DeviceEvent::Key(raw_key_event) => {
                    game.key_event(raw_key_event);
                }
                _ => {}
            },
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
                    let fps_99th = 1.0 / render_time_stats.percentile_99();
                    log::info!("FPS: {:.0} ({:.0} ± {:.0})", fps_99th, fps, fps_std);
                }
            }
            _ => {}
        })
        .unwrap();
}
