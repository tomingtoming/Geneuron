use macroquad::prelude::*;
use nalgebra as na;
use ::std::f32::consts::PI;

/// Camera that handles view transformations, zooming and following objects
pub struct Camera {
    // Core camera properties
    pub position: na::Point2<f32>,        // Current camera position
    pub target_position: na::Point2<f32>, // Target position for smooth movement
    pub zoom: f32,                        // Current zoom level
    pub target_zoom: f32,                 // Target zoom for smooth zooming
    
    // Viewport properties
    pub viewport_width: f32,
    pub viewport_height: f32,
    
    // Zoom constraints and behavior
    pub min_zoom: f32,
    pub max_zoom: f32,
    #[allow(dead_code)]
    pub zoom_speed: f32,  // Kept for future zoom acceleration features
    
    // Following behavior
    is_following: bool,
    follow_target_position: Option<na::Point2<f32>>,
    #[allow(dead_code)]
    follow_smoothness: f32,  // Will be used for smooth camera following
    
    // World properties
    world_bounds: (f32, f32),
    
    // Enhanced camera features
    damping: f32,             // Movement smoothing factor (0.0-1.0)
    zoom_damping: f32,        // Zoom smoothing factor
    shake_duration: f32,      // Current duration of camera shake
    shake_intensity: f32,     // Intensity of camera shake
    #[allow(dead_code)]
    shake_decay: f32,     // How quickly shake effect decays (for future screen shake effects)
    shake_offset: na::Vector2<f32>, // Current shake offset
    
    // Focus points system
    focus_points: Vec<(na::Point2<f32>, f32)>, // Vec of (position, weight)
    
    // Minimap properties
    #[allow(dead_code)]
    minimap_enabled: bool,  // Planned minimap feature
    #[allow(dead_code)]
    minimap_size: f32,      // Size as percentage of screen (0.0-1.0)
    #[allow(dead_code)]
    minimap_position: (f32, f32), // Position in normalized screen coordinates
    #[allow(dead_code)]
    minimap_zoom: f32,      // Zoom level for minimap
}

impl Camera {
    /// Create a new camera with default settings
    pub fn new(viewport_width: f32, viewport_height: f32, world_bounds: (f32, f32)) -> Self {
        // Calculate optimal default zoom based on world size
        let width_ratio = viewport_width / world_bounds.0;
        let height_ratio = viewport_height / world_bounds.1;
        let default_zoom = (width_ratio.min(height_ratio) * 0.8).max(0.33);

        let default_position = na::Point2::new(
            world_bounds.0 / 2.0 - viewport_width / (2.0 * default_zoom),
            world_bounds.1 / 2.0 - viewport_height / (2.0 * default_zoom),
        );
        
        Camera {
            position: default_position,
            target_position: default_position,
            zoom: default_zoom,
            target_zoom: default_zoom,
            viewport_width,
            viewport_height,
            min_zoom: 0.2,      // Allow zooming out further
            max_zoom: 8.0,      // Allow zooming in more
            zoom_speed: 0.15,   // Slightly faster zoom
            is_following: false,
            follow_target_position: None,
            follow_smoothness: 0.08, // Smoother following
            world_bounds,
            damping: 0.85,      // Smooth movement
            zoom_damping: 0.9,  // Smooth zooming
            shake_duration: 0.0,
            shake_intensity: 0.0,
            shake_decay: 5.0,
            shake_offset: na::Vector2::new(0.0, 0.0),
            focus_points: Vec::new(),
            minimap_enabled: true,
            minimap_size: 0.15, // 15% of screen width
            minimap_position: (0.95, 0.95), // Bottom right corner
            minimap_zoom: 0.1,  // Very zoomed out for overview
        }
    }

    /// Set the viewport dimensions (e.g. when window resizes)
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        if (self.viewport_width - width).abs() > f32::EPSILON || 
           (self.viewport_height - height).abs() > f32::EPSILON {
            
            // Save old viewport center in world coordinates
            let old_center_x = self.position.x + self.viewport_width / (2.0 * self.zoom);
            let old_center_y = self.position.y + self.viewport_height / (2.0 * self.zoom);

            // Update viewport dimensions
            self.viewport_width = width;
            self.viewport_height = height;

            // Adjust position to maintain the same center point
            self.position.x = old_center_x - width / (2.0 * self.zoom);
            self.position.y = old_center_y - height / (2.0 * self.zoom);
            self.target_position = self.position;

            // Ensure camera stays within bounds
            self.constrain_camera();
        }
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> na::Point2<f32> {
        // Get base position
        let base_pos = na::Point2::new(
            self.position.x + screen_pos.x / self.zoom,
            self.position.y + screen_pos.y / self.zoom,
        );
        
        // Remove shake offset to get accurate world position
        na::Point2::new(
            base_pos.x - self.shake_offset.x / self.zoom,
            base_pos.y - self.shake_offset.y / self.zoom,
        )
    }
    
    /// Convert world coordinates to screen coordinates
    #[allow(dead_code)]
    pub fn world_to_screen(&self, world_pos: na::Point2<f32>) -> Vec2 {
        Vec2::new(
            (world_pos.x - self.position.x) * self.zoom + self.shake_offset.x,
            (world_pos.y - self.position.y) * self.zoom + self.shake_offset.y,
        )
    }

    /// Handle mouse wheel zoom with enhanced position preservation
    pub fn handle_mouse_wheel_zoom(&mut self, delta: f32, mouse_pos: Option<Vec2>) {
        // Skip if delta is too small
        if delta.abs() < 0.001 {
            return;
        }
        
        // Get world coordinates at mouse position or screen center
        let target_pos = if let Some(pos) = mouse_pos {
            // Convert screen coordinates to world coordinates
            self.screen_to_world(pos)
        } else {
            // Use screen center if mouse position is not provided
            let screen_center = Vec2::new(self.viewport_width * 0.5, self.viewport_height * 0.5);
            self.screen_to_world(screen_center)
        };

        // Calculate new zoom level with exponential scaling for smoother zoom
        let zoom_factor = if delta > 0.0 { 1.1 } else { 0.9 };
        let new_zoom = (self.target_zoom * zoom_factor).clamp(self.min_zoom, self.max_zoom);
        
        // Only adjust camera position if zoom actually changed
        if (new_zoom - self.target_zoom).abs() > f32::EPSILON {
            // Calculate how the target point would move in screen space after zooming
            let zoom_ratio = new_zoom / self.target_zoom;
            
            // Calculate offset from target to camera in world space
            let offset_x = target_pos.x - self.target_position.x;
            let offset_y = target_pos.y - self.target_position.y;
            
            // Adjust target camera position to keep the mouse position fixed
            self.target_position.x = target_pos.x - offset_x / zoom_ratio;
            self.target_position.y = target_pos.y - offset_y / zoom_ratio;
            
            // Set the new target zoom
            self.target_zoom = new_zoom;
            
            // Apply camera constraints to target position
            self.constrain_target();
        }
    }

    /// Set target zoom level with smooth transition
    pub fn set_zoom(&mut self, zoom: f32) {
        self.target_zoom = zoom.clamp(self.min_zoom, self.max_zoom);
    }

    /// Update camera position and effects
    pub fn update(&mut self, dt: f32) {
        // Process camera shake
        self.update_camera_shake(dt);
        
        // Process focus points if any
        self.update_focus_points();
        
        // Handle smooth zoom transition with improved damping
        if (self.zoom - self.target_zoom).abs() > 0.0001 {
            self.zoom = self.zoom + (self.target_zoom - self.zoom) * (1.0 - self.zoom_damping) * (60.0 * dt);
        } else {
            self.zoom = self.target_zoom;
        }
        
        // Follow target with improved smoothing
        if self.is_following && self.follow_target_position.is_some() {
            let target = self.follow_target_position.unwrap();
            
            // Calculate the position of the viewport centered on the target
            let view_width = self.viewport_width / self.zoom;
            let view_height = self.viewport_height / self.zoom;
            
            // Dynamic zoom based on target velocity (if available)
            if let Some(target_velocity) = self.get_target_velocity() {
                let speed = target_velocity.norm();
                if speed > 50.0 {
                    // Gradually zoom out for faster moving targets
                    let desired_zoom = (self.target_zoom * 0.99).max(self.min_zoom);
                    self.target_zoom = self.target_zoom + (desired_zoom - self.target_zoom) * 0.01;
                } else if speed < 10.0 {
                    // Gradually zoom in for slower targets
                    let desired_zoom = (self.target_zoom * 1.01).min(self.max_zoom * 0.7);
                    self.target_zoom = self.target_zoom + (desired_zoom - self.target_zoom) * 0.005;
                }
            }
            
            self.target_position.x = target.x - view_width / 2.0;
            self.target_position.y = target.y - view_height / 2.0;
            
            // Make sure target position is within constraints
            self.constrain_target();
        }
        
        // Apply smooth movement to camera position
        if (self.position - self.target_position).norm() > 0.01 {
            // Consider wrapping in toroidal world for shortest path
            let dx = Self::shortest_distance(self.position.x, self.target_position.x, self.world_bounds.0);
            let dy = Self::shortest_distance(self.position.y, self.target_position.y, self.world_bounds.1);
            
            // Smooth camera movement with improved damping
            self.position.x += dx * (1.0 - self.damping) * (60.0 * dt);
            self.position.y += dy * (1.0 - self.damping) * (60.0 * dt);
            
            // Handle world wrapping
            self.position.x = self.position.x.rem_euclid(self.world_bounds.0);
            self.position.y = self.position.y.rem_euclid(self.world_bounds.1);
        } else {
            self.position = self.target_position.clone();
        }
        
        // Final constraint check
        self.constrain_camera();
    }
    
    /// Set the follow target position
    pub fn set_follow_target(&mut self, position: Option<na::Point2<f32>>) {
        self.follow_target_position = position;
    }
    
    /// Enable or disable following mode
    pub fn set_following(&mut self, following: bool) {
        self.is_following = following && self.follow_target_position.is_some();
    }
    
    /// Toggle the following state
    pub fn toggle_following(&mut self) -> bool {
        if self.follow_target_position.is_some() {
            self.is_following = !self.is_following;
        } else {
            self.is_following = false;
        }
        self.is_following
    }
    
    /// Check if camera is following a target
    pub fn is_following(&self) -> bool {
        self.is_following
    }
    
    /// Apply camera movement directly (e.g., from drag operations)
    pub fn move_by(&mut self, dx: f32, dy: f32) {
        // Apply inverse movement (dragging right moves camera left)
        self.target_position.x -= dx / self.zoom;
        self.target_position.y -= dy / self.zoom;
        
        // Handle world wrapping in a more predictable way
        // First, normalize the positions to be within a positive range
        // to avoid unexpected behavior with rem_euclid on negative values
        let world_width = self.world_bounds.0;
        let world_height = self.world_bounds.1;
        
        // Wrap around using modulo but handle the transition across the world bounds carefully
        while self.target_position.x < 0.0 {
            self.target_position.x += world_width;
        }
        while self.target_position.x >= world_width {
            self.target_position.x -= world_width;
        }
        
        while self.target_position.y < 0.0 {
            self.target_position.y += world_height;
        }
        while self.target_position.y >= world_height {
            self.target_position.y -= world_height;
        }
        
        // Ensure camera stays within bounds
        self.constrain_target();
        
        // Disable following if manually moved
        self.is_following = false;
    }

    /// Center the camera on the world and reset zoom
    pub fn reset_view(&mut self) {
        // Reset zoom to default value that shows a good portion of the world
        let width_ratio = self.viewport_width / self.world_bounds.0;
        let height_ratio = self.viewport_height / self.world_bounds.1;
        self.target_zoom = (width_ratio.min(height_ratio) * 0.8).max(0.33);
        
        // Center camera on the world
        self.target_position.x = self.world_bounds.0 / 2.0 - self.viewport_width / (2.0 * self.target_zoom);
        self.target_position.y = self.world_bounds.1 / 2.0 - self.viewport_height / (2.0 * self.target_zoom);
        
        // Immediately snap position to target for a true reset
        self.position.x = self.target_position.x;
        self.position.y = self.target_position.y;
        
        // Disable following
        self.is_following = false;
        
        // Clear focus points
        self.clear_focus_points();
        
        // Clear any shake
        self.shake_duration = 0.0;
        self.shake_offset = na::Vector2::new(0.0, 0.0);
        
        // Ensure camera stays within bounds
        self.constrain_target();
    }

    /// Apply world boundary constraints to the camera
    pub fn constrain_camera(&mut self) {
        // Calculate visible area dimensions in world coordinates
        let visible_width = self.viewport_width / self.zoom;
        let visible_height = self.viewport_height / self.zoom;
        
        // Calculate maximum allowed camera offsets
        let max_x = self.world_bounds.0 - visible_width * 0.5;
        let min_x = -visible_width * 0.5;
        
        let max_y = self.world_bounds.1 - visible_height * 0.5;
        let min_y = -visible_height * 0.5;
        
        // Constrain camera position
        self.position.x = self.position.x.clamp(min_x, max_x);
        self.position.y = self.position.y.clamp(min_y, max_y);
    }
    
    /// Constrain target position
    fn constrain_target(&mut self) {
        // Calculate visible area dimensions in world coordinates
        let visible_width = self.viewport_width / self.target_zoom;
        let visible_height = self.viewport_height / self.target_zoom;
        
        // Calculate maximum allowed camera offsets
        let max_x = self.world_bounds.0 - visible_width * 0.5;
        let min_x = -visible_width * 0.5;
        
        let max_y = self.world_bounds.1 - visible_height * 0.5;
        let min_y = -visible_height * 0.5;
        
        // Constrain target position
        self.target_position.x = self.target_position.x.clamp(min_x, max_x);
        self.target_position.y = self.target_position.y.clamp(min_y, max_y);
    }

    /// Get the macroquad Camera2D for rendering
    #[allow(dead_code)]
    pub fn get_macroquad_camera(&self) -> Camera2D {
        Camera2D {
            zoom: vec2(
                2.0 / self.viewport_width * self.zoom,
                2.0 / self.viewport_height * self.zoom,
            ),
            target: vec2(
                self.position.x + self.viewport_width / (2.0 * self.zoom) + self.shake_offset.x / self.zoom,
                self.position.y + self.viewport_height / (2.0 * self.zoom) + self.shake_offset.y / self.zoom,
            ),
            ..Default::default()
        }
    }
    
    /// Start a camera shake effect
    #[allow(dead_code)]
    pub fn shake(&mut self, intensity: f32, duration: f32) {
        self.shake_intensity = intensity;
        self.shake_duration = duration;
    }
    
    /// Update camera shake effect
    fn update_camera_shake(&mut self, dt: f32) {
        if self.shake_duration > 0.0 {
            // Decrease shake duration
            self.shake_duration -= dt;
            
            // Calculate shake intensity with decay
            let current_intensity = if self.shake_duration <= 0.0 {
                self.shake_intensity = 0.0;
                self.shake_duration = 0.0;
                0.0
            } else {
                self.shake_intensity * (self.shake_duration.min(0.5) / 0.5)
            };
            
            // Generate random shake offset
            let angle = ::rand::random::<f32>() * PI * 2.0;
            let distance = current_intensity * ::rand::random::<f32>() * self.viewport_width * 0.02;
            
            self.shake_offset.x = angle.cos() * distance;
            self.shake_offset.y = angle.sin() * distance;
        }
    }
    
    /// Add a focus point for the camera to track
    #[allow(dead_code)]
    pub fn add_focus_point(&mut self, position: na::Point2<f32>, weight: f32) {
        self.focus_points.push((position, weight));
    }
    
    /// Clear all focus points
    pub fn clear_focus_points(&mut self) {
        self.focus_points.clear();
    }
    
    /// Update camera position based on focus points
    fn update_focus_points(&mut self) {
        if self.is_following || self.focus_points.is_empty() {
            return;
        }
        
        // Calculate weighted average of focus points
        let mut total_weight = 0.0;
        let mut target_x = 0.0;
        let mut target_y = 0.0;
        
        for (point, weight) in &self.focus_points {
            total_weight += weight;
            target_x += point.x * weight;
            target_y += point.y * weight;
        }
        
        if total_weight > 0.0 {
            target_x /= total_weight;
            target_y /= total_weight;
            
            // Calculate the position of the viewport centered on the target
            let view_width = self.viewport_width / self.zoom;
            let view_height = self.viewport_height / self.zoom;
            
            self.target_position.x = target_x - view_width / 2.0;
            self.target_position.y = target_y - view_height / 2.0;
            
            // Apply constraints
            self.constrain_target();
        }
    }
    
    /// Check if there are any active focus points
    pub fn has_focus_points(&self) -> bool {
        !self.focus_points.is_empty()
    }

    /// Draw the minimap
    #[allow(dead_code)]
    pub fn draw_minimap(&self, world: &crate::world::World) {
        if !self.minimap_enabled {
            return;
        }
        
        // Calculate minimap dimensions and position
        let minimap_width = self.viewport_width * self.minimap_size;
        let minimap_height = minimap_width * (self.world_bounds.1 / self.world_bounds.0);
        
        // Calculate minimap position (bottom right corner by default)
        let minimap_x = self.viewport_width * self.minimap_position.0 - minimap_width;
        let minimap_y = self.viewport_height * self.minimap_position.1 - minimap_height;
        
        // Draw minimap background
        draw_rectangle(
            minimap_x, 
            minimap_y, 
            minimap_width, 
            minimap_height, 
            Color::new(0.0, 0.0, 0.0, 0.7)
        );
        
        // Calculate scaling factors for minimap
        let scale_x = minimap_width / self.world_bounds.0;
        let scale_y = minimap_height / self.world_bounds.1;
        
        // Draw food on minimap (as small dots)
        for food in &world.food_manager.foods {
            let dot_x = minimap_x + food.position.x * scale_x;
            let dot_y = minimap_y + food.position.y * scale_y;
            draw_circle(dot_x, dot_y, 1.0, GREEN);
        }
        
        // Draw creatures on minimap (as small dots)
        for creature in &world.creatures {
            // Use different colors for selected creatures
            let dot_color = if self.follow_target_position.is_some() && 
               self.is_following &&
               self.follow_target_position.unwrap() == creature.physics.position {
                YELLOW
            } else {
                creature.color
            };
            
            let dot_x = minimap_x + creature.physics.position.x * scale_x;
            let dot_y = minimap_y + creature.physics.position.y * scale_y;
            draw_circle(dot_x, dot_y, 1.5, dot_color);
        }
        
        // Draw current viewport area on minimap
        let view_x = minimap_x + self.position.x * scale_x;
        let view_y = minimap_y + self.position.y * scale_y;
        let view_width = (self.viewport_width / self.zoom) * scale_x;
        let view_height = (self.viewport_height / self.zoom) * scale_y;
        
        draw_rectangle_lines(view_x, view_y, view_width, view_height, 1.0, YELLOW);
        
        // Draw focus points if any
        for (point, weight) in &self.focus_points {
            let point_x = minimap_x + point.x * scale_x;
            let point_y = minimap_y + point.y * scale_y;
            let size = 2.0 + weight * 2.0;
            draw_circle(point_x, point_y, size, ORANGE);
        }
        
        // Draw border around minimap
        draw_rectangle_lines(minimap_x, minimap_y, minimap_width, minimap_height, 1.0, WHITE);
        
        // Draw minimap label
        draw_text(
            "MINIMAP",
            minimap_x + 5.0,
            minimap_y + 15.0,
            12.0,
            WHITE
        );
    }
    
    /// Toggle minimap visibility
    #[allow(dead_code)]
    pub fn toggle_minimap(&mut self) {
        self.minimap_enabled = !self.minimap_enabled;
    }
    
    /// Get velocity of the followed target from the world (if available)
    #[allow(dead_code)]
    pub fn get_target_velocity(&self) -> Option<na::Vector2<f32>> {
        // This would need to be implemented by checking the creature's velocity
        // For now just return None
        None
    }
    
    /// Smooth transition to a new target position
    #[allow(dead_code)]
    pub fn focus_on(&mut self, position: na::Point2<f32>, zoom_level: Option<f32>) {
        let view_width = self.viewport_width / self.zoom;
        let view_height = self.viewport_height / self.zoom;
        
        self.target_position.x = position.x - view_width / 2.0;
        self.target_position.y = position.y - view_height / 2.0;
        
        if let Some(zoom) = zoom_level {
            self.target_zoom = zoom.clamp(self.min_zoom, self.max_zoom);
        }
        
        self.constrain_target();
    }
    
    /// Get minimap status
    #[allow(dead_code)]
    pub fn is_minimap_enabled(&self) -> bool {
        self.minimap_enabled
    }
    
    /// Set minimap position
    #[allow(dead_code)]
    pub fn set_minimap_position(&mut self, position: (f32, f32)) {
        self.minimap_position = position;
    }
    
    /// Set minimap size
    #[allow(dead_code)]
    pub fn set_minimap_size(&mut self, size: f32) {
        self.minimap_size = size.clamp(0.05, 0.3);
    }

    /// Calculate shortest distance in a wrapped (toroidal) world
    fn shortest_distance(from: f32, to: f32, world_size: f32) -> f32 {
        let direct = to - from;
        let wrapped = if direct > 0.0 {
            direct - world_size
        } else {
            direct + world_size
        };
        
        if direct.abs() < wrapped.abs() {
            direct
        } else {
            wrapped
        }
    }
}
