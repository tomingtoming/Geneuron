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

struct GameState {
    world: world::World,
    renderer: rendering::Renderer,
    paused: bool,
}

impl GameState {
    fn new() -> Self {
        GameState {
            world: world::World::new(WORLD_WIDTH, WORLD_HEIGHT),
            renderer: rendering::Renderer::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            paused: false,
        }
    }

    async fn update(&mut self) {
        // Toggle pause with space key
        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
        }

        // Smooth zoom control
        if is_key_down(KeyCode::Z) {
            self.renderer.set_zoom((self.renderer.zoom * 1.05).min(5.0)); // Limit max zoom
        }
        if is_key_down(KeyCode::X) {
            self.renderer.set_zoom((self.renderer.zoom * 0.95).max(0.2)); // Limit min zoom
        }

        // Toggle follow mode with F key
        if is_key_pressed(KeyCode::F) {
            self.renderer.toggle_follow();
        }

        // Select creature with left mouse click
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse_pos = mouse_position();
            let world_pos = na::Point2::new(
                mouse_pos.0 / self.renderer.zoom,
                mouse_pos.1 / self.renderer.zoom,
            );
            self.select_creature_at(world_pos);
        }

        // Deselect creature with right mouse click
        if is_mouse_button_pressed(MouseButton::Right) {
            self.renderer.select_creature(None);
        }

        if !self.paused {
            self.world.update(get_frame_time());
        }

        self.renderer.resize(screen_width(), screen_height());
    }

    fn select_creature_at(&mut self, position: na::Point2<f32>) {
        // Convert window coordinates to world coordinates
        let world_x = self.renderer.camera_offset.x + position.x / self.renderer.zoom;
        let world_y = self.renderer.camera_offset.y + position.y / self.renderer.zoom;
        let world_pos = na::Point2::new(world_x, world_y);

        let selected_index = self
            .world
            .creatures
            .iter()
            .enumerate()
            .filter(|(_, creature)| {
                // Distance calculation considering toroidal structure
                creature
                    .physics
                    .distance_to(&world_pos, self.world.world_bounds)
                    < 20.0 // Increase selection range
            })
            .map(|(index, _)| index)
            .next();
        self.renderer.select_creature(selected_index);
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Geneuron-RS".to_owned(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        window_resizable: true,
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
