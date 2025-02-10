mod neural;
mod creature;
mod physics;
mod world;
mod rendering;
mod food;

use ggez::{Context, GameResult};
use ggez::event::{self, EventHandler};
use ggez::winit::event::VirtualKeyCode;
use nalgebra as na;

// Window constants
const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

struct GameState {
    world: world::World,
    renderer: rendering::Renderer,
    paused: bool,
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Toggle pause with space key
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Space) {
            self.paused = !self.paused;
        }

        // Smooth zoom control
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Z) {
            self.renderer.set_zoom((self.renderer.zoom * 1.05).min(5.0));  // Limit max zoom
        }
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::X) {
            self.renderer.set_zoom((self.renderer.zoom * 0.95).max(0.2));  // Limit min zoom
        }

        // Toggle follow mode with F key
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::F) {
            self.renderer.toggle_follow();
        }

        // Select creature with left mouse click
        if ctx.mouse.button_pressed(ggez::input::mouse::MouseButton::Left) {
            let mouse_pos = ctx.mouse.position();
            let world_mouse_pos = na::Point2::new(
                mouse_pos.x * self.renderer.zoom,
                mouse_pos.y * self.renderer.zoom,
            );
            self.select_creature_at(world_mouse_pos);
        }

        // Deselect creature with right mouse click
        if ctx.mouse.button_pressed(ggez::input::mouse::MouseButton::Right) {
            self.renderer.select_creature(None);
        }

        if !self.paused {
            let dt = ctx.time.delta().as_secs_f32();
            self.world.update(dt);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.renderer.render(ctx, &self.world)
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.renderer.resize(width, height);
        Ok(())
    }
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        let (width, height) = ctx.gfx.drawable_size();
        Ok(GameState {
            world: world::World::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            renderer: rendering::Renderer::new(width, height),
            paused: false,
        })
    }

    fn select_creature_at(&mut self, position: na::Point2<f32>) {
        // Convert screen coordinates to world coordinates
        let world_pos = na::Point2::new(
            self.renderer.camera_offset.x + position.x / self.renderer.zoom,
            self.renderer.camera_offset.y + position.y / self.renderer.zoom,
        );

        let selected_index = self.world.creatures.iter()
            .enumerate()
            .filter(|(_, creature)| na::distance(&creature.physics.position, &world_pos) < 10.0)
            .map(|(index, _)| index)
            .next();
        self.renderer.select_creature(selected_index);
    }
}

fn main() -> GameResult {
    // Game configuration
    let cb = ggez::ContextBuilder::new("geneuron", "neuroevolution")
        .window_setup(ggez::conf::WindowSetup::default().title("Geneuron-RS"))
        .window_mode(ggez::conf::WindowMode::default()
            .dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)
            .resizable(true));
    
    let (mut ctx, event_loop) = cb.build()?;
    
    // Create and run game state
    let state = GameState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
