use crate::world::World;
use macroquad::prelude::*;
use nalgebra::Point2;

pub struct Renderer {
    window_size: (f32, f32),
    pub zoom: f32,                    // Make zoom field public
    selected_creature: Option<usize>, // Add selected creature index
    pub camera_offset: Point2<f32>,   // カメラの位置をパブリックに
    following_selected: bool,         // 選択中の生物を追従するかどうか
}

impl Renderer {
    pub fn new(width: f32, height: f32) -> Self {
        Renderer {
            window_size: (width, height),
            zoom: 0.5, // デフォルトのズームを1.0から0.5に変更（より広い視野）
            selected_creature: None,
            camera_offset: Point2::new(0.0, 0.0),
            following_selected: false,
        }
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        // より広い範囲でズーム可能に
        self.zoom = zoom.clamp(0.2, 2.0); // max zoom を5.0から2.0に変更
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        // 古いビューポート範囲を保存
        let old_view_width = self.window_size.0 / self.zoom;
        let old_view_height = self.window_size.1 / self.zoom;
        let old_center_x = self.camera_offset.x + old_view_width / 2.0;
        let old_center_y = self.camera_offset.y + old_view_height / 2.0;

        // ウィンドウサイズを更新
        self.window_size = (width, height);

        // 新しいビューポートの中心を古いビューポートの中心に合わせる
        let new_view_width = width / self.zoom;
        let new_view_height = height / self.zoom;
        self.camera_offset.x = old_center_x - new_view_width / 2.0;
        self.camera_offset.y = old_center_y - new_view_height / 2.0;
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
                    let view_width = self.window_size.0 / self.zoom;
                    let view_height = self.window_size.1 / self.zoom;

                    // 生物を中心にしたいビューポートの位置を計算
                    let target_x = creature.physics.position.x - view_width / 2.0;
                    let target_y = creature.physics.position.y - view_height / 2.0;

                    // カメラを必要最小限だけ移動させる
                    let dx = if target_x < self.camera_offset.x {
                        let diff = self.camera_offset.x - target_x;
                        if diff > world.world_bounds.0 / 2.0 {
                            world.world_bounds.0 - diff
                        } else {
                            -diff
                        }
                    } else if target_x > self.camera_offset.x {
                        let diff = target_x - self.camera_offset.x;
                        if diff > world.world_bounds.0 / 2.0 {
                            -(world.world_bounds.0 - diff)
                        } else {
                            diff
                        }
                    } else {
                        0.0
                    };

                    let dy = if target_y < self.camera_offset.y {
                        let diff = self.camera_offset.y - target_y;
                        if diff > world.world_bounds.1 / 2.0 {
                            world.world_bounds.1 - diff
                        } else {
                            -diff
                        }
                    } else if target_y > self.camera_offset.y {
                        let diff = target_y - self.camera_offset.y;
                        if diff > world.world_bounds.1 / 2.0 {
                            -(world.world_bounds.1 - diff)
                        } else {
                            diff
                        }
                    } else {
                        0.0
                    };

                    // カメラ位置を更新
                    self.camera_offset.x =
                        (self.camera_offset.x + dx).rem_euclid(world.world_bounds.0);
                    self.camera_offset.y =
                        (self.camera_offset.y + dy).rem_euclid(world.world_bounds.1);
                }
            }
        }
    }

    fn draw_wrapped_circle(
        &self,
        pos: Point2<f32>,
        radius: f32,
        color: Color,
        world_bounds: (f32, f32),
    ) {
        let view_left = self.camera_offset.x;
        let view_right = self.camera_offset.x + self.window_size.0 / self.zoom;
        let view_top = self.camera_offset.y;
        let view_bottom = self.camera_offset.y + self.window_size.1 / self.zoom;

        let positions = [
            (pos.x, pos.y),
            (pos.x - world_bounds.0, pos.y),
            (pos.x + world_bounds.0, pos.y),
            (pos.x, pos.y - world_bounds.1),
            (pos.x, pos.y + world_bounds.1),
            (pos.x - world_bounds.0, pos.y - world_bounds.1),
            (pos.x - world_bounds.0, pos.y + world_bounds.1),
            (pos.x + world_bounds.0, pos.y - world_bounds.1),
            (pos.x + world_bounds.0, pos.y + world_bounds.1),
        ];

        for &(x, y) in &positions {
            if x >= view_left - radius
                && x <= view_right + radius
                && y >= view_top - radius
                && y <= view_bottom + radius
            {
                draw_circle(x, y, radius, color);
            }
        }
    }

    fn draw_wrapped_line(
        &self,
        start: Point2<f32>,
        end: Point2<f32>,
        thickness: f32,
        color: Color,
        world_bounds: (f32, f32),
    ) {
        let view_left = self.camera_offset.x;
        let view_right = self.camera_offset.x + self.window_size.0 / self.zoom;
        let view_top = self.camera_offset.y;
        let view_bottom = self.camera_offset.y + self.window_size.1 / self.zoom;

        let offsets = [
            (0.0, 0.0),
            (-world_bounds.0, 0.0),
            (world_bounds.0, 0.0),
            (0.0, -world_bounds.1),
            (0.0, world_bounds.1),
            (-world_bounds.0, -world_bounds.1),
            (-world_bounds.0, world_bounds.1),
            (world_bounds.0, -world_bounds.1),
            (world_bounds.0, world_bounds.1),
        ];

        for &(dx, dy) in &offsets {
            let s = Point2::new(start.x + dx, start.y + dy);
            let e = Point2::new(end.x + dx, end.y + dy);

            if (s.x >= view_left || e.x >= view_left)
                && (s.x <= view_right || e.x <= view_right)
                && (s.y >= view_top || e.y >= view_top)
                && (s.y <= view_bottom || e.y <= view_bottom)
            {
                draw_line(s.x, s.y, e.x, e.y, thickness, color);
            }
        }
    }

    pub async fn render(&mut self, world: &World) {
        self.update_camera(world);

        // Set camera
        set_camera(&Camera2D {
            zoom: vec2(2.0 / self.window_size.0 * self.zoom, 2.0 / self.window_size.1 * self.zoom),
            target: vec2(
                self.camera_offset.x + self.window_size.0 / (2.0 * self.zoom),
                self.camera_offset.y + self.window_size.1 / (2.0 * self.zoom),
            ),
            ..Default::default()
        });

        clear_background(BLACK);

        // Draw viewport border
        draw_rectangle_lines(
            self.camera_offset.x,
            self.camera_offset.y,
            self.window_size.0 / self.zoom,
            self.window_size.1 / self.zoom,
            2.0,
            YELLOW,
        );

        // Draw food sources
        for food in &world.food_manager.foods {
            self.draw_wrapped_circle(food.position, food.size, food.color, world.world_bounds);
        }

        // Draw creatures
        for (i, creature) in world.creatures.iter().enumerate() {
            // Creature body
            self.draw_wrapped_circle(
                creature.physics.position,
                10.0,
                creature.color,
                world.world_bounds,
            );

            // Direction indicator
            let end_pos = Point2::new(
                creature.physics.position.x + 20.0 * creature.physics.rotation.cos(),
                creature.physics.position.y + 20.0 * creature.physics.rotation.sin(),
            );
            self.draw_wrapped_line(
                creature.physics.position,
                end_pos,
                2.0,
                creature.mode_color,
                world.world_bounds,
            );

            // Highlight selected creature
            if let Some(selected_index) = self.selected_creature {
                if selected_index == i {
                    self.draw_wrapped_circle(
                        creature.physics.position,
                        12.0,
                        YELLOW,
                        world.world_bounds,
                    );

                    // Display creature info
                    let details = format!(
                        "Energy: {:.2}\nAge: {:.2}\nFitness: {:.2}\nState: {:?}\nSpeed: {:.2}\nPosition: ({:.0}, {:.0})\nGender: {:?}",
                        creature.physics.energy,
                        creature.age,
                        creature.fitness,
                        creature.behavior_state,
                        creature.physics.velocity.norm(),
                        creature.physics.position.x,
                        creature.physics.position.y,
                        creature.gender,
                    );

                    draw_text(
                        &details,
                        self.camera_offset.x + 10.0,
                        self.camera_offset.y + 30.0,
                        24.0,
                        WHITE,
                    );

                    if self.following_selected {
                        draw_text(
                            "Following",
                            self.camera_offset.x + 10.0,
                            self.camera_offset.y + 120.0,
                            24.0,
                            GREEN,
                        );
                    }
                }
            }
        }

        // Status info
        let status = format!(
            "Generation: {}\nPopulation: {}\nTime: {:.1}s\nFPS: {}",
            world.generation,
            world.creatures.len(),
            world.elapsed_time,
            get_fps(),
        );
        draw_text(
            &status,
            self.camera_offset.x + 30.0,
            self.camera_offset.y + 50.0,
            28.0,
            WHITE,
        );

        // Selected creature details
        if let Some(selected_index) = self.selected_creature {
            if let Some(creature) = world.creatures.get(selected_index) {
                let details = format!(
                    "Selected Creature\n---------------\nEnergy: {:.2}\nAge: {:.2}\nFitness: {:.2}\nState: {:?}\nSpeed: {:.2}\nPosition: ({:.0}, {:.0})\nGender: {:?}\n---------------\n{}",
                    creature.physics.energy,
                    creature.age,
                    creature.fitness,
                    creature.behavior_state,
                    creature.physics.velocity.norm(),
                    creature.physics.position.x,
                    creature.physics.position.y,
                    creature.gender,
                    if self.following_selected { "[Following]" } else { "" }
                );

                // Semi-transparent background
                draw_rectangle(
                    self.camera_offset.x + self.window_size.0 / self.zoom - 300.0,
                    self.camera_offset.y + 20.0,
                    280.0,
                    300.0,
                    Color::new(0.0, 0.0, 0.0, 0.7),
                );

                draw_text(
                    &details,
                    self.camera_offset.x + self.window_size.0 / self.zoom - 280.0,
                    self.camera_offset.y + 30.0,
                    24.0,
                    WHITE,
                );
            }
        }
    }
}
