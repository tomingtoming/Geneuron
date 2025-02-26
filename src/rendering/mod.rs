use crate::world::World;
use macroquad::prelude::*;
use nalgebra::Point2;
use crate::camera::Camera;

pub struct Renderer {
    window_size: (f32, f32),
    pub zoom: f32,                    
    pub selected_creature: Option<usize>,
    pub camera_offset: Point2<f32>,   
    following_selected: bool,         
    // Track hovered creature
    hovered_creature: Option<usize>,
    // Track hover creature ID
    hover_creature_id: Option<usize>,
}

impl Renderer {
    pub fn new(width: f32, height: f32) -> Self {
        Renderer {
            window_size: (width, height),
            zoom: 0.5, 
            selected_creature: None,
            camera_offset: Point2::new(0.0, 0.0),
            following_selected: false,
            hovered_creature: None,
            hover_creature_id: None,
        }
    }

    pub fn select_creature(&mut self, index: Option<usize>) {
        self.selected_creature = index;
        if index.is_none() {
            self.following_selected = false;
        }
    }

    pub async fn render(&self, world: &World, camera: &Camera) {
        // Draw world grid for better navigation
        self.draw_grid(world.world_bounds);

        // Draw food sources
        for food in &world.food_manager.foods {
            self.draw_wrapped_circle(food.position, food.size, food.color, world.world_bounds);
        }

        // Draw creatures
        for (i, creature) in world.creatures.iter().enumerate() {
            // Creature body
            let is_selected = self.selected_creature == Some(i);
            let is_hovered = self.hover_creature_id == Some(i);
            
            // Draw selection highlight first (underneath creature)
            if is_selected {
                self.draw_wrapped_circle(
                    creature.physics.position,
                    14.0,
                    YELLOW,
                    world.world_bounds,
                );
            } else if is_hovered {
                // Hover effect
                self.draw_wrapped_circle(
                    creature.physics.position,
                    12.0,
                    Color::new(0.5, 0.5, 0.5, 0.7),
                    world.world_bounds,
                );
            }
            
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

            // Energy indicator as a ring around creature
            self.draw_energy_ring(creature, world.world_bounds);
        }

        // Status info with semi-transparent background
        self.draw_status_info(world, camera);

        // Show detailed info for selected creature
        self.draw_creature_details(world, camera);
    }
    
    fn draw_wrapped_circle(
        &self,
        pos: Point2<f32>,
        radius: f32,
        color: Color,
        world_bounds: (f32, f32),
    ) {
        // Get camera view bounds
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
    
    fn draw_energy_ring(&self, creature: &crate::creature::Creature, world_bounds: (f32, f32)) {
        // Draw energy level as ring around creature
        let energy_normalized = creature.physics.energy.clamp(0.0, 1.0);
        let energy_color = if energy_normalized < 0.3 {
            RED 
        } else if energy_normalized < 0.7 {
            GOLD
        } else {
            GREEN
        };
        
        // Draw as a circle arc
        let start_angle = 0.0;
        let end_angle = std::f32::consts::PI * 2.0 * energy_normalized;
        
        let positions = [
            (creature.physics.position.x, creature.physics.position.y),
            (creature.physics.position.x - world_bounds.0, creature.physics.position.y),
            (creature.physics.position.x + world_bounds.0, creature.physics.position.y),
            (creature.physics.position.x, creature.physics.position.y - world_bounds.1),
            (creature.physics.position.x, creature.physics.position.y + world_bounds.1),
            (creature.physics.position.x - world_bounds.0, creature.physics.position.y - world_bounds.1),
            (creature.physics.position.x - world_bounds.0, creature.physics.position.y + world_bounds.1),
            (creature.physics.position.x + world_bounds.0, creature.physics.position.y - world_bounds.1),
            (creature.physics.position.x + world_bounds.0, creature.physics.position.y + world_bounds.1),
        ];
        
        for &(x, y) in &positions {
            let view_left = self.camera_offset.x;
            let view_right = self.camera_offset.x + self.window_size.0 / self.zoom;
            let view_top = self.camera_offset.y;
            let view_bottom = self.camera_offset.y + self.window_size.1 / self.zoom;
            
            if x >= view_left - 15.0 && x <= view_right + 15.0 &&
               y >= view_top - 15.0 && y <= view_bottom + 15.0 {
                draw_circle_lines(x, y, 13.0, 2.0, energy_color);
                
                // Draw arc representing energy level
                let segments = (end_angle * 10.0) as usize;
                if segments > 0 {
                    for i in 0..segments {
                        let segment_start = start_angle + (end_angle * i as f32 / segments as f32);
                        let segment_end = start_angle + (end_angle * (i + 1) as f32 / segments as f32);
                        
                        let start_x = x + 13.0 * segment_start.cos();
                        let start_y = y + 13.0 * segment_start.sin();
                        let end_x = x + 13.0 * segment_end.cos();
                        let end_y = y + 13.0 * segment_end.sin();
                        
                        draw_line(start_x, start_y, end_x, end_y, 2.0, energy_color);
                    }
                }
            }
        }
    }
    
    fn draw_grid(&self, world_bounds: (f32, f32)) {
        let grid_size = 200.0;  // Size of grid cells
        let grid_color = Color::new(0.2, 0.2, 0.2, 0.5);  // Dark gray, semi-transparent
        
        // Calculate grid boundaries
        let view_left = self.camera_offset.x;
        let view_right = self.camera_offset.x + self.window_size.0 / self.zoom;
        let view_top = self.camera_offset.y;
        let view_bottom = self.camera_offset.y + self.window_size.1 / self.zoom;
        
        // Calculate start/end grid lines
        let start_x = (view_left / grid_size).floor() * grid_size;
        let end_x = (view_right / grid_size).ceil() * grid_size;
        let start_y = (view_top / grid_size).floor() * grid_size;
        let end_y = (view_bottom / grid_size).ceil() * grid_size;
        
        // Draw vertical grid lines
        let mut x = start_x;
        while x <= end_x {
            let wrapped_x = x.rem_euclid(world_bounds.0);
            draw_line(wrapped_x, view_top, wrapped_x, view_bottom, 1.0, grid_color);
            x += grid_size;
        }
        
        // Draw horizontal grid lines
        let mut y = start_y;
        while y <= end_y {
            let wrapped_y = y.rem_euclid(world_bounds.1);
            draw_line(view_left, wrapped_y, view_right, wrapped_y, 1.0, grid_color);
            y += grid_size;
        }
    }
    
    fn draw_status_info(&self, world: &World, camera: &Camera) {
        // Semi-transparent background for status info
        draw_rectangle(
            camera.position.x + 10.0,
            camera.position.y + 10.0,
            220.0,
            120.0,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
        
        let follow_status = if camera.is_following() {
            "Following creature"
        } else if camera.has_focus_points() {
            "Tracking top creatures"
        } else {
            "Free camera"
        };
        
        let status = format!(
            "Generation: {}\nPopulation: {}\nTime: {:.1}s\nFPS: {}\nZoom: {:.1}x\n{}",
            world.generation,
            world.creatures.len(),
            world.elapsed_time,
            get_fps(),
            camera.zoom,
            follow_status
        );
        
        draw_text(
            &status,
            camera.position.x + 20.0,
            camera.position.y + 35.0,
            20.0,
            WHITE,
        );
    }
    
    #[allow(dead_code)]
    fn draw_controls_help(&self, _camera: &Camera) {
        // Implementation for future controls help display
    }
    
    fn draw_creature_details(&self, world: &World, camera: &Camera) {
        if let Some(selected_index) = self.selected_creature {
            if let Some(creature) = world.creatures.get(selected_index) {
                let energy_status = if creature.physics.energy < 0.3 {
                    "Low Energy"
                } else if creature.physics.energy < 0.7 {
                    "Moderate Energy"
                } else {
                    "High Energy"
                };
                
                let details = format!(
                    "Selected Creature #{}\n---------------\nEnergy: {:.2} ({})\nAge: {:.1}\nFitness: {:.1}\nState: {:?}\nSpeed: {:.1}\nPosition: ({:.0}, {:.0})\nGender: {:?}\n",
                    selected_index,
                    creature.physics.energy,
                    energy_status,
                    creature.age,
                    creature.fitness,
                    creature.behavior_state,
                    creature.physics.velocity.norm(),
                    creature.physics.position.x,
                    creature.physics.position.y,
                    creature.gender,
                );

                // Semi-transparent background
                draw_rectangle(
                    camera.position.x + camera.viewport_width / camera.zoom - 280.0,
                    camera.position.y + 20.0,
                    260.0,
                    300.0,
                    Color::new(0.0, 0.0, 0.0, 0.7),
                );

                draw_text(
                    &details,
                    camera.position.x + camera.viewport_width / camera.zoom - 270.0,
                    camera.position.y + 40.0,
                    18.0,  // Slightly smaller font
                    WHITE,
                );
                
                // Add follow status indicator
                if camera.is_following() {
                    draw_text(
                        "[Following]",
                        camera.position.x + camera.viewport_width / camera.zoom - 270.0,
                        camera.position.y + 260.0,
                        20.0,
                        YELLOW,
                    );
                }
            }
        }
    }

    pub fn set_hover_creature(&mut self, creature_id: Option<usize>) {
        self.hover_creature_id = creature_id;
        self.hovered_creature = creature_id;
    }
}
