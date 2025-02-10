use ggez::{Context, GameResult};
use ggez::graphics::{self, Canvas, Color, DrawParam, Mesh, Text};
use nalgebra::Point2;
use crate::world::World;

pub struct Renderer {
    window_size: (f32, f32),
    pub zoom: f32,  // Make zoom field public
    selected_creature: Option<usize>,  // Add selected creature index
    pub camera_offset: Point2<f32>,  // カメラの位置をパブリックに
    following_selected: bool,         // 選択中の生物を追従するかどうか
}

impl Renderer {
    pub fn new(width: f32, height: f32) -> Self {
        Renderer {
            window_size: (width, height),
            zoom: 1.0,
            selected_creature: None,
            camera_offset: Point2::new(0.0, 0.0),
            following_selected: false,
        }
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.window_size = (width, height);
    }

    pub fn select_creature(&mut self, index: Option<usize>) {
        self.selected_creature = index;
        if index.is_none() {
            self.following_selected = false;
        }
    }

    pub fn toggle_follow(&mut self) {
        if self.selected_creature.is_some() {
            self.following_selected = !self.following_selected;
        }
    }

    fn update_camera(&mut self, world: &World) {
        if self.following_selected {
            if let Some(selected_idx) = self.selected_creature {
                if let Some(creature) = world.creatures.get(selected_idx) {
                    // カメラを選択中の生物の位置に設定
                    self.camera_offset = Point2::new(
                        creature.physics.position.x - self.window_size.0 / (2.0 * self.zoom),
                        creature.physics.position.y - self.window_size.1 / (2.0 * self.zoom)
                    );
                }
            }
        }
    }

    pub fn render(&mut self, ctx: &mut Context, world: &World) -> GameResult {
        self.update_camera(world);
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
        
        // カメラオフセットを考慮したビューポート設定
        canvas.set_screen_coordinates(graphics::Rect::new(
            self.camera_offset.x, 
            self.camera_offset.y, 
            self.window_size.0 / self.zoom, 
            self.window_size.1 / self.zoom,
        ));

        // Draw food sources
        for food in &world.food_manager.foods {
            let food_circle = Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [food.position.x, food.position.y],
                food.size,
                0.1,
                food.color,
            )?;
            canvas.draw(&food_circle, DrawParam::default());
        }

        // Draw creatures
        for (i, creature) in world.creatures.iter().enumerate() {
            // Creature body
            let body = Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [creature.physics.position.x, creature.physics.position.y],
                10.0,
                0.1,
                creature.color,
            )?;
            canvas.draw(&body, DrawParam::default());

            // Direction indicator with mode color
            let direction_line = Mesh::new_line(
                ctx,
                &[
                    [creature.physics.position.x, creature.physics.position.y],
                    [
                        creature.physics.position.x + 20.0 * creature.physics.rotation.cos(),
                        creature.physics.position.y + 20.0 * creature.physics.rotation.sin()
                    ],
                ],
                2.0,
                creature.mode_color,
            )?;
            canvas.draw(&direction_line, DrawParam::default());

            // Highlight and show details for selected creature
            if let Some(selected_index) = self.selected_creature {
                if selected_index == i {
                    // Highlight circle
                    let highlight_circle = Mesh::new_circle(
                        ctx,
                        graphics::DrawMode::stroke(2.0),
                        [creature.physics.position.x, creature.physics.position.y],
                        12.0,
                        0.1,
                        Color::YELLOW,
                    )?;
                    canvas.draw(&highlight_circle, DrawParam::default());

                    // Display detailed creature information
                    let details = format!(
                        "Energy: {:.2}\n\
                         Age: {:.2}\n\
                         Fitness: {:.2}\n\
                         State: {:?}\n\
                         Speed: {:.2}\n\
                         Position: ({:.0}, {:.0})\n\
                         Gender: {:?}",
                        creature.physics.energy,
                        creature.age,
                        creature.fitness,
                        creature.behavior_state,
                        creature.physics.velocity.norm(),
                        creature.physics.position.x,
                        creature.physics.position.y,
                        creature.gender,
                    );

                    // 画面端に固定された位置に情報を表示
                    let details_text = Text::new(details);
                    canvas.draw(
                        &details_text,
                        DrawParam::default()
                            .color(Color::WHITE)
                            .dest([
                                self.camera_offset.x + 10.0,
                                self.camera_offset.y + 10.0
                            ]),
                    );

                    // 追従状態の表示
                    if self.following_selected {
                        let following_text = Text::new("Following");
                        canvas.draw(
                            &following_text,
                            DrawParam::default()
                                .color(Color::GREEN)
                                .dest([
                                    self.camera_offset.x + 10.0,
                                    self.camera_offset.y + 120.0
                                ]),
                        );
                    }
                }
            }
        }

        // Display simulation information (画面の右上に固定)
        let info_text = Text::new(format!(
            "Generation: {}\nCreatures: {}\nElapsed Time: {:.1}s\nFPS: {:.1}",
            world.generation,
            world.creatures.len(),
            world.elapsed_time,
            ctx.time.fps(),
        ));
        canvas.draw(
            &info_text,
            DrawParam::default()
                .color(Color::WHITE)
                .dest([
                    self.camera_offset.x + self.window_size.0 / self.zoom - 150.0,
                    self.camera_offset.y + 10.0
                ]),
        );

        canvas.finish(ctx)?;
        Ok(())
    }
}