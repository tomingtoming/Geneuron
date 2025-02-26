use crate::world::World;
use macroquad::prelude::*;
use nalgebra::Point2;

pub struct Renderer {
    window_size: (f32, f32),
    pub zoom: f32,                    
    selected_creature: Option<usize>, 
    pub camera_offset: Point2<f32>,   
    following_selected: bool,         
    // Add new fields for improved rendering
    hovered_creature: Option<usize>,  // Track hovered creature
    pub is_dragging: bool,                // Track if user is currently dragging
    last_mouse_pos: (f32, f32),       // Store last mouse position for drag calculation
    target_zoom: f32,                 // Target zoom level for smooth zooming
    zoom_transition_speed: f32,       // How quickly zoom transitions to target
    hover_creature_id: Option<usize>, // Track hover creature ID
}

impl Renderer {
    pub fn new(width: f32, height: f32) -> Self {
        Renderer {
            window_size: (width, height),
            zoom: 0.5, 
            selected_creature: None,
            camera_offset: Point2::new(0.0, 0.0),
            following_selected: false,
            // Initialize new fields
            hovered_creature: None,
            is_dragging: false,
            last_mouse_pos: (0.0, 0.0),
            target_zoom: 0.5,    // Match initial zoom
            zoom_transition_speed: 8.0,  // Adjust for faster/slower transitions
            hover_creature_id: None,
        }
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        // Limit zoom between 0.33 (see entire world) and 5.0 (detailed view)
        self.target_zoom = zoom.clamp(0.33, 5.0);
    }

    pub fn update(&mut self, world: &World, dt: f32) {
        // Store previous mouse position to calculate accurate deltas
        // Remove this unused variable declaration
        // let prev_mouse_pos = self.last_mouse_pos;
        
        // Keep only this properly prefixed version
        let _prev_mouse_pos = self.last_mouse_pos;
        
        // Update current mouse position
        let current_mouse_pos = mouse_position();
        
        // Process mouse wheel for zoom
        let wheel_movement = mouse_wheel().1;
        if wheel_movement != 0.0 {
            // Get current mouse position for zoom centering
            let mouse_pos = current_mouse_pos;
            
            // Calculate world point under cursor before zoom
            let world_x = self.camera_offset.x + mouse_pos.0 / self.zoom;
            let world_y = self.camera_offset.y + mouse_pos.1 / self.zoom;
            
            // Adjust target zoom based on wheel direction (smoother than direct change)
            let zoom_factor = if wheel_movement > 0.0 { 1.1 } else { 0.9 };
            self.target_zoom = (self.target_zoom * zoom_factor).clamp(0.33, 5.0);
            
            // After changing zoom, adjust camera to keep mouse position over same world point
            if !self.following_selected {
                let new_x = world_x - mouse_pos.0 / self.target_zoom;
                let new_y = world_y - mouse_pos.1 / self.target_zoom;
                self.camera_offset.x = new_x;
                self.camera_offset.y = new_y;
            }
        }
        
        // Smooth zoom transition
        if self.zoom != self.target_zoom {
            self.zoom += (self.target_zoom - self.zoom) * dt * self.zoom_transition_speed;
            
            // Snap to target when very close to avoid floating point issues
            if (self.zoom - self.target_zoom).abs() < 0.001 {
                self.zoom = self.target_zoom;
            }
        }
        
        // Handle dragging - improved for reliable behavior in all window sizes
        // Start drag on middle mouse button press or shift+left click
        if (is_mouse_button_pressed(MouseButton::Middle) || 
            (is_mouse_button_pressed(MouseButton::Left) && is_key_down(KeyCode::LeftShift))) && 
           !self.is_dragging {
            self.is_dragging = true;
            // Initialize last_mouse_pos only when drag starts
            self.last_mouse_pos = current_mouse_pos;
            self.following_selected = false; // Disable follow mode when dragging
        }
        
        // End drag on button release
        if (is_mouse_button_released(MouseButton::Middle) || 
            (is_mouse_button_released(MouseButton::Left) && is_key_down(KeyCode::LeftShift))) && 
           self.is_dragging {
            self.is_dragging = false;
        }
        
        // Process drag movement with improved calculations
        if self.is_dragging {
            // Calculate real screen space movement delta
            let dx = (current_mouse_pos.0 - self.last_mouse_pos.0) / self.zoom;
            let dy = (current_mouse_pos.1 - self.last_mouse_pos.1) / self.zoom;
            
            // Update camera offset - move in opposite direction of mouse movement
            self.camera_offset.x -= dx;
            self.camera_offset.y -= dy;
            
            // Update last mouse position for next frame's calculation
            self.last_mouse_pos = current_mouse_pos;
            
            // Apply toroidal wrapping to camera offset
            self.camera_offset.x = self.camera_offset.x.rem_euclid(world.world_bounds.0);
            self.camera_offset.y = self.camera_offset.y.rem_euclid(world.world_bounds.1);
        } else {
            // Keep track of mouse position even when not dragging
            self.last_mouse_pos = current_mouse_pos;
        }
        
        // Update hover state - Fix: use current_mouse_pos instead of undefined mouse_pos
        self.update_hover_state(current_mouse_pos, world);
        
        // Update camera position for selected creature
        self.update_camera(world);
    }
    
    fn update_hover_state(&mut self, mouse_pos: (f32, f32), world: &World) {
        // Convert mouse position to world coordinates
        let world_x = self.camera_offset.x + mouse_pos.0 / self.zoom;
        let world_y = self.camera_offset.y + mouse_pos.1 / self.zoom;
        let world_pos = Point2::new(world_x, world_y);
        
        // Find creature under mouse cursor
        self.hovered_creature = world.creatures.iter()
            .enumerate()
            .filter(|(_, creature)| {
                creature.physics.distance_to(&world_pos, world.world_bounds) < 15.0
            })
            .map(|(index, _)| index)
            .next();
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        // Only process if there's an actual change in dimensions
        if (self.window_size.0 - width).abs() < 0.1 && 
           (self.window_size.1 - height).abs() < 0.1 {
            return;
        }
        
        // Save old viewport center in world coordinates
        let old_center_x = self.camera_offset.x + self.window_size.0 / (2.0 * self.zoom);
        let old_center_y = self.camera_offset.y + self.window_size.1 / (2.0 * self.zoom);

        // Update window size
        self.window_size = (width, height);

        // Recalculate camera offset to maintain the same center point
        self.camera_offset.x = old_center_x - width / (2.0 * self.zoom);
        self.camera_offset.y = old_center_y - height / (2.0 * self.zoom);
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

                    // Calculate the position of the viewport centered on the creature
                    let target_x = creature.physics.position.x - view_width / 2.0;
                    let target_y = creature.physics.position.y - view_height / 2.0;

                    // Smooth camera movement - gradually move toward target position
                    let dx = target_x - self.camera_offset.x;
                    let dy = target_y - self.camera_offset.y;
                    
                    // Wrap around for shortest path in toroidal world
                    let wrapped_dx = if dx.abs() > world.world_bounds.0 / 2.0 {
                        if dx > 0.0 { dx - world.world_bounds.0 } else { dx + world.world_bounds.0 }
                    } else {
                        dx
                    };
                    
                    let wrapped_dy = if dy.abs() > world.world_bounds.1 / 2.0 {
                        if dy > 0.0 { dy - world.world_bounds.1 } else { dy + world.world_bounds.1 }
                    } else {
                        dy
                    };
                    
                    // Apply smooth movement
                    self.camera_offset.x += wrapped_dx * 0.05;
                    self.camera_offset.y += wrapped_dy * 0.05;
                    
                    // Handle world wrapping
                    self.camera_offset.x = self.camera_offset.x.rem_euclid(world.world_bounds.0);
                    self.camera_offset.y = self.camera_offset.y.rem_euclid(world.world_bounds.1);
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

    pub async fn render(&self, world: &World) {
        // Set camera
        set_camera(&Camera2D {
            zoom: vec2(
                2.0 / self.window_size.0 * self.zoom,
                2.0 / self.window_size.1 * self.zoom,
            ),
            target: vec2(
                self.camera_offset.x + self.window_size.0 / (2.0 * self.zoom),
                self.camera_offset.y + self.window_size.1 / (2.0 * self.zoom),
            ),
            ..Default::default()
        });

        clear_background(BLACK);

        // Draw world grid for better navigation
        self.draw_grid(world.world_bounds);

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
            let is_selected = self.selected_creature == Some(i);
            let is_hovered = self.hovered_creature == Some(i);
            
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
        self.draw_status_info(world);

        // Display help text for controls
        self.draw_controls_help();

        // Show detailed info for selected creature
        self.draw_creature_details(world);
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
    
    fn draw_status_info(&self, world: &World) {
        // Semi-transparent background for status info
        draw_rectangle(
            self.camera_offset.x + 10.0,
            self.camera_offset.y + 10.0,
            220.0,
            100.0,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
        
        let status = format!(
            "Generation: {}\nPopulation: {}\nTime: {:.1}s\nFPS: {}",
            world.generation,
            world.creatures.len(),
            world.elapsed_time,
            get_fps(),
        );
        
        draw_text(
            &status,
            self.camera_offset.x + 20.0,
            self.camera_offset.y + 35.0,
            24.0,
            WHITE,
        );
    }
    
    fn draw_controls_help(&self) {
        // Draw controls help in bottom left
        let controls_text = "Controls:\nZ/X or Mouse Wheel: Zoom\nSpace: Pause\nF: Follow selected\nLeft Click: Select\nRight Click: Deselect\nShift+Drag: Move camera";
        
        draw_rectangle(
            self.camera_offset.x + 10.0,
            self.camera_offset.y + self.window_size.1 / self.zoom - 140.0,
            220.0,
            130.0,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
        
        draw_text(
            controls_text,
            self.camera_offset.x + 20.0,
            self.camera_offset.y + self.window_size.1 / self.zoom - 120.0,
            16.0,
            WHITE,
        );
    }
    
    fn draw_creature_details(&self, world: &World) {
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
                    self.camera_offset.x + self.window_size.0 / self.zoom - 280.0,
                    self.camera_offset.y + 20.0,
                    260.0,
                    300.0,
                    Color::new(0.0, 0.0, 0.0, 0.7),
                );

                draw_text(
                    &details,
                    self.camera_offset.x + self.window_size.0 / self.zoom - 270.0,
                    self.camera_offset.y + 40.0,
                    20.0,  // Slightly smaller font
                    WHITE,
                );
            }
        }
    }

    pub fn set_hover_creature(&mut self, creature_id: Option<usize>) {
        self.hover_creature_id = creature_id;
    }
}
