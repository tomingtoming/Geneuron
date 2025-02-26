mod camera;
mod creature;
mod food;
mod neural;
mod physics;
mod rendering;
mod world;

use macroquad::prelude::*;
use nalgebra as na;

// Window constants
const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

// World constants
const WORLD_WIDTH: f32 = 2400.0; // 3 times the window width
const WORLD_HEIGHT: f32 = 1800.0; // 3 times the window height

// UI Constants
const SELECTION_THRESHOLD: f32 = 20.0; // Distance threshold for selecting creatures
const HOVER_THRESHOLD: f32 = 25.0;     // Distance threshold for hover effect

struct GameState {
    world: world::World,
    renderer: rendering::Renderer,
    camera: camera::Camera, // New camera instance
    paused: bool,
    last_mouse_pos: (f32, f32),
    hover_creature_id: Option<usize>,
    is_dragging: bool,
}

impl GameState {
    fn new() -> Self {
        // Create camera with appropriate world bounds
        let camera = camera::Camera::new(WINDOW_WIDTH, WINDOW_HEIGHT, (WORLD_WIDTH, WORLD_HEIGHT));
        
        GameState {
            world: world::World::new(WORLD_WIDTH, WORLD_HEIGHT),
            renderer: rendering::Renderer::new(WINDOW_WIDTH, WINDOW_HEIGHT), // Pass window dimensions
            camera,
            paused: false,
            last_mouse_pos: (0.0, 0.0),
            hover_creature_id: None,
            is_dragging: false,
        }
    }

    async fn update(&mut self) {
        let dt = get_frame_time();
        self.last_mouse_pos = mouse_position();

        // Toggle pause with space key
        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
        }

        // Smooth zoom control with keyboard
        if is_key_down(KeyCode::Z) {
            self.camera.set_zoom(self.camera.zoom * 1.05);
        }
        if is_key_down(KeyCode::X) {
            self.camera.set_zoom(self.camera.zoom * 0.95);
        }

        // Mouse wheel zoom control with focus on cursor position
        let mouse_wheel = mouse_wheel();
        if mouse_wheel.1 != 0.0 {
            let delta = mouse_wheel.1 * 0.1; // Scale factor for smoother zoom
            self.camera.handle_mouse_wheel_zoom(delta, Some(Vec2::new(self.last_mouse_pos.0, self.last_mouse_pos.1)));
        }

        // Reset view with R key
        if is_key_pressed(KeyCode::R) {
            self.camera.reset_view();
        }

        // Toggle follow mode with F key
        if is_key_pressed(KeyCode::F) {
            if let Some(selected_idx) = self.renderer.selected_creature {
                if let Some(creature) = self.world.creatures.get(selected_idx) {
                    self.camera.set_follow_target(Some(creature.physics.position));
                    self.camera.toggle_following();
                }
            } else {
                self.camera.set_following(false);
            }
        }

        // Select creature with left mouse click
        if is_mouse_button_pressed(MouseButton::Left) && 
           !is_key_down(KeyCode::LeftShift) && 
           !self.is_dragging {
            let world_pos = self.camera.screen_to_world(Vec2::new(self.last_mouse_pos.0, self.last_mouse_pos.1));
            self.select_creature_at(world_pos);
        }

        // Deselect creature with right mouse click
        if is_mouse_button_pressed(MouseButton::Right) {
            self.renderer.select_creature(None);
            self.camera.set_following(false);
        }
        
        // Handle camera dragging - with middle mouse or shift+left click
        if (is_mouse_button_pressed(MouseButton::Middle) || 
            (is_mouse_button_pressed(MouseButton::Left) && is_key_down(KeyCode::LeftShift))) && 
           !self.is_dragging {
            self.is_dragging = true;
            self.last_mouse_pos = mouse_position();
        }
        
        if (is_mouse_button_released(MouseButton::Middle) || 
            (is_mouse_button_released(MouseButton::Left) && is_key_down(KeyCode::LeftShift))) && 
           self.is_dragging {
            self.is_dragging = false;
        }
        
        if self.is_dragging {
            let current_mouse_pos = mouse_position();
            let dx = current_mouse_pos.0 - self.last_mouse_pos.0;
            let dy = current_mouse_pos.1 - self.last_mouse_pos.1;
            
            self.camera.move_by(dx, dy);
            self.last_mouse_pos = current_mouse_pos;
        }
        
        // Update hover state for creature under cursor
        self.update_hover_creature();

        // Update camera (follows selected creature if in follow mode)
        self.camera.update(dt);
        
        // Update camera target position if following a creature
        if self.camera.is_following() {
            if let Some(selected_idx) = self.renderer.selected_creature {
                if let Some(creature) = self.world.creatures.get(selected_idx) {
                    self.camera.set_follow_target(Some(creature.physics.position));
                }
            }
        }

        // Update simulation if not paused
        if !self.paused {
            self.world.update(dt);
        }

        // Update viewport size if window resized
        self.camera.set_viewport_size(screen_width(), screen_height());
    }

    fn update_hover_creature(&mut self) {
        let world_pos = self.camera.screen_to_world(Vec2::new(self.last_mouse_pos.0, self.last_mouse_pos.1));
        
        // Adjust hover threshold based on zoom level
        let threshold = HOVER_THRESHOLD / self.camera.zoom;
        
        self.hover_creature_id = self.world.creatures.iter()
            .enumerate()
            .filter(|(_, creature)| {
                creature.physics.distance_to(&world_pos, self.world.world_bounds) < threshold
            })
            .map(|(index, _)| index)
            .next();
            
        // Pass hover state to renderer
        self.renderer.set_hover_creature(self.hover_creature_id);
    }

    fn select_creature_at(&mut self, position: na::Point2<f32>) {
        // Adjust selection threshold based on zoom level
        let threshold = SELECTION_THRESHOLD / self.camera.zoom;
        
        let selected_index = self.world.creatures.iter()
            .enumerate()
            .filter(|(_, creature)| {
                creature.physics.distance_to(&position, self.world.world_bounds) < threshold
            })
            .map(|(index, _)| index)
            .next();
            
        self.renderer.select_creature(selected_index);
        
        // Update camera follow target if a creature is selected
        if let Some(idx) = selected_index {
            if let Some(creature) = self.world.creatures.get(idx) {
                self.camera.set_follow_target(Some(creature.physics.position));
            }
        } else {
            self.camera.set_follow_target(None);
            self.camera.set_following(false);
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Geneuron-RS".to_owned(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = GameState::new();

    loop {
        state.update().await;
        state.renderer.render(&state.world, &state.camera).await;
        next_frame().await;
    }
}
