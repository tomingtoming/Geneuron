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
// Zoom limits - min allows seeing most of the world, max for detailed view
const MIN_ZOOM: f32 = 0.5; // Allows seeing the entire world (1/3 of window size)
const MAX_ZOOM: f32 = 5.0;  // Maximum zoom for detailed inspection

struct GameState {
    world: world::World,
    renderer: rendering::Renderer,
    paused: bool,
    last_mouse_pos: (f32, f32),
    hover_creature_id: Option<usize>,
}

impl GameState {
    fn new() -> Self {
        GameState {
            world: world::World::new(WORLD_WIDTH, WORLD_HEIGHT),
            renderer: rendering::Renderer::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            paused: false,
            last_mouse_pos: (0.0, 0.0),
            hover_creature_id: None,
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
            self.zoom_at((self.renderer.zoom * 1.05).min(MAX_ZOOM), None);
        }
        if is_key_down(KeyCode::X) {
            self.zoom_at((self.renderer.zoom * 0.95).max(MIN_ZOOM), None);
        }

        // Mouse wheel zoom control with focus on cursor position
        let mouse_wheel = mouse_wheel();
        if mouse_wheel.1 != 0.0 {
            let zoom_factor = if mouse_wheel.1 > 0.0 { 1.1 } else { 0.9 };
            let new_zoom = (self.renderer.zoom * zoom_factor).clamp(MIN_ZOOM, MAX_ZOOM);
            self.zoom_at(new_zoom, Some(self.last_mouse_pos));
        }

        // Reset view with R key
        if is_key_pressed(KeyCode::R) {
            self.reset_view();
        }

        // Toggle follow mode with F key
        if is_key_pressed(KeyCode::F) {
            self.renderer.toggle_follow();
        }

        // Select creature with left mouse click (not during renderer's drag state)
        if is_mouse_button_pressed(MouseButton::Left) && 
           !is_key_down(KeyCode::LeftShift) && 
           !self.renderer.is_dragging {
            let world_pos = self.screen_to_world(self.last_mouse_pos);
            self.select_creature_at(world_pos);
        }

        // Deselect creature with right mouse click
        if is_mouse_button_pressed(MouseButton::Right) {
            self.renderer.select_creature(None);
        }
        
        // Update hover state for creature under cursor
        self.update_hover_creature();

        // Update renderer first (handles input)
        self.renderer.update(&self.world, dt);
        
        // Ensure camera stays within bounds after any movement
        self.constrain_camera();

        // Update simulation if not paused
        if !self.paused {
            self.world.update(dt);
        }

        self.renderer.resize(screen_width(), screen_height());
    }

    fn zoom_at(&mut self, new_zoom: f32, focus_point: Option<(f32, f32)>) {
        let old_zoom = self.renderer.zoom;
        self.renderer.set_zoom(new_zoom);
        
        // If we have a focus point, adjust camera offset to zoom toward that point
        if let Some((focus_x, focus_y)) = focus_point {
            // Calculate the world position of the focus point before zoom
            let world_x = self.renderer.camera_offset.x + focus_x / old_zoom;
            let world_y = self.renderer.camera_offset.y + focus_y / old_zoom;
            
            // Calculate new camera offset to maintain focus point
            self.renderer.camera_offset.x = world_x - focus_x / new_zoom;
            self.renderer.camera_offset.y = world_y - focus_y / new_zoom;
        }
        
        // Constrain camera position to ensure the world is always visible
        self.constrain_camera();
    }
    
    // Constrain camera position to ensure world bounds remain visible
    fn constrain_camera(&mut self) {
        // Calculate visible area dimensions in world coordinates
        let visible_width = screen_width() / self.renderer.zoom;
        let visible_height = screen_height() / self.renderer.zoom;
        
        // Calculate maximum allowed camera offsets
        // Adjust based on actual world_bounds type (tuple instead of rectangle)
        let max_x = self.world.world_bounds.0 - visible_width * 0.5;
        let min_x = -visible_width * 0.5;
        
        let max_y = self.world.world_bounds.1 - visible_height * 0.5;
        let min_y = -visible_height * 0.5;
        
        // Constrain camera position
        self.renderer.camera_offset.x = self.renderer.camera_offset.x.clamp(min_x, max_x);
        self.renderer.camera_offset.y = self.renderer.camera_offset.y.clamp(min_y, max_y);
    }
    
    fn screen_to_world(&self, screen_pos: (f32, f32)) -> na::Point2<f32> {
        na::Point2::new(
            self.renderer.camera_offset.x + screen_pos.0 / self.renderer.zoom,
            self.renderer.camera_offset.y + screen_pos.1 / self.renderer.zoom,
        )
    }
    
    fn update_hover_creature(&mut self) {
        let world_pos = self.screen_to_world(self.last_mouse_pos);
        
        // Adjust hover threshold based on zoom level
        let threshold = HOVER_THRESHOLD / self.renderer.zoom;
        
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
        let threshold = SELECTION_THRESHOLD / self.renderer.zoom;
        
        let selected_index = self.world.creatures.iter()
            .enumerate()
            .filter(|(_, creature)| {
                creature.physics.distance_to(&position, self.world.world_bounds) < threshold
            })
            .map(|(index, _)| index)
            .next();
            
        self.renderer.select_creature(selected_index);
    }
    
    fn reset_view(&mut self) {
        // Reset zoom to default value that shows a good portion of the world
        self.renderer.set_zoom(1.0);
        
        // Center camera on the world (adjust for world_bounds being a tuple)
        self.renderer.camera_offset.x = self.world.world_bounds.0 / 2.0 - screen_width() / 2.0 / self.renderer.zoom;
        self.renderer.camera_offset.y = self.world.world_bounds.1 / 2.0 - screen_height() / 2.0 / self.renderer.zoom;
        
        // Reset selection and follow state
        self.renderer.select_creature(None);
        // Use toggle_follow(false) instead of set_follow_mode which doesn't exist
        self.renderer.toggle_follow();
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
        state.renderer.render(&state.world).await;
        next_frame().await;
    }
}
