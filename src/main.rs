mod neural;
mod creature;
mod physics;
mod world;
mod rendering;
mod food;

use ggez::{Context, GameResult};
use ggez::event::{self, EventHandler};
use ggez::winit::event::VirtualKeyCode;

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

        // Zoom control
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Z) {
            self.renderer.set_zoom(self.renderer.zoom * 1.05);
        }
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::X) {
            self.renderer.set_zoom(self.renderer.zoom * 0.95);
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
