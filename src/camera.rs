use macroquad::prelude::*;
use nalgebra as na;

/// Camera that handles view transformations, zooming and following objects
pub struct Camera {
    pub position: na::Point2<f32>,
    pub zoom: f32,
    target_zoom: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub zoom_speed: f32,
    is_following: bool,
    follow_target_position: Option<na::Point2<f32>>,
    follow_smoothness: f32,
    world_bounds: (f32, f32),
}

impl Camera {
    /// Create a new camera with default settings
    pub fn new(viewport_width: f32, viewport_height: f32, world_bounds: (f32, f32)) -> Self {
        Camera {
            position: na::Point2::new(0.0, 0.0),
            zoom: 1.0,
            target_zoom: 1.0,
            viewport_width,
            viewport_height,
            min_zoom: 0.33, // Minimum zoom to see the entire world
            max_zoom: 5.0,  // Maximum zoom for detailed inspection
            zoom_speed: 0.1, // Speed of zoom when using scroll wheel
            is_following: false,
            follow_target_position: None,
            follow_smoothness: 0.05,
            world_bounds,
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

            // Ensure camera stays within bounds
            self.constrain_camera();
        }
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> na::Point2<f32> {
        na::Point2::new(
            self.position.x + screen_pos.x / self.zoom,
            self.position.y + screen_pos.y / self.zoom,
        )
    }

    /// Handle mouse wheel zoom with position preservation
    pub fn handle_mouse_wheel_zoom(&mut self, delta: f32, mouse_pos: Option<Vec2>) {
        // Get world coordinates at mouse position or screen center
        let target_pos = if let Some(pos) = mouse_pos {
            // Convert screen coordinates to world coordinates
            self.screen_to_world(pos)
        } else {
            // Use screen center if mouse position is not provided
            let screen_center = Vec2::new(self.viewport_width * 0.5, self.viewport_height * 0.5);
            self.screen_to_world(screen_center)
        };

        // Save current zoom value
        let old_zoom = self.zoom;
        
        // Change zoom value
        self.zoom += delta * self.zoom_speed;
        self.zoom = self.zoom.clamp(self.min_zoom, self.max_zoom);
        
        // Only adjust camera position if zoom actually changed
        if (old_zoom - self.zoom).abs() > f32::EPSILON {
            // Important: Calculate how far the target was from the camera before zooming
            let offset = target_pos - self.position;
            
            // Calculate zoom ratio
            let zoom_ratio = self.zoom / old_zoom;
            
            // Adjust camera position after zoom - maintain appropriate distance from target position
            self.position = target_pos - offset * zoom_ratio;
        }

        // Apply camera constraints
        self.constrain_camera();
    }

    /// Set target zoom level with smooth transition
    pub fn set_zoom(&mut self, zoom: f32) {
        self.target_zoom = zoom.clamp(self.min_zoom, self.max_zoom);
    }

    /// Update camera position to follow target if enabled
    pub fn update(&mut self, dt: f32) {
        // Handle smooth zoom transition
        if (self.zoom - self.target_zoom).abs() > f32::EPSILON {
            self.zoom += (self.target_zoom - self.zoom) * dt * 8.0; // Smooth transition
            
            // Snap to target when very close to avoid floating point issues
            if (self.zoom - self.target_zoom).abs() < 0.001 {
                self.zoom = self.target_zoom;
            }
            
            // Recalculate constraints after zoom changes
            self.constrain_camera();
        }
        
        // Follow target if enabled
        if self.is_following && self.follow_target_position.is_some() {
            let target = self.follow_target_position.unwrap();
            let view_width = self.viewport_width / self.zoom;
            let view_height = self.viewport_height / self.zoom;

            // Calculate the position of the viewport centered on the target
            let target_x = target.x - view_width / 2.0;
            let target_y = target.y - view_height / 2.0;

            // Smooth camera movement - gradually move toward target position
            let dx = target_x - self.position.x;
            let dy = target_y - self.position.y;
            
            // Wrap around for shortest path in toroidal world
            let wrapped_dx = if dx.abs() > self.world_bounds.0 / 2.0 {
                if dx > 0.0 { dx - self.world_bounds.0 } else { dx + self.world_bounds.0 }
            } else {
                dx
            };
            
            let wrapped_dy = if dy.abs() > self.world_bounds.1 / 2.0 {
                if dy > 0.0 { dy - self.world_bounds.1 } else { dy + self.world_bounds.1 }
            } else {
                dy
            };
            
            // Apply smooth movement
            self.position.x += wrapped_dx * self.follow_smoothness;
            self.position.y += wrapped_dy * self.follow_smoothness;
            
            // Handle world wrapping
            self.position.x = self.position.x.rem_euclid(self.world_bounds.0);
            self.position.y = self.position.y.rem_euclid(self.world_bounds.1);
            
            // Apply constraints
            self.constrain_camera();
        }
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
        self.position.x -= dx / self.zoom;
        self.position.y -= dy / self.zoom;
        
        // Handle world wrapping
        self.position.x = self.position.x.rem_euclid(self.world_bounds.0);
        self.position.y = self.position.y.rem_euclid(self.world_bounds.1);
        
        // Ensure camera stays within bounds
        self.constrain_camera();
        
        // Disable following if manually moved
        self.is_following = false;
    }

    /// Center the camera on the world and reset zoom
    pub fn reset_view(&mut self) {
        // Reset zoom to default value that shows a good portion of the world
        self.zoom = 1.0;
        self.target_zoom = 1.0;
        
        // Center camera on the world
        self.position.x = self.world_bounds.0 / 2.0 - self.viewport_width / 2.0 / self.zoom;
        self.position.y = self.world_bounds.1 / 2.0 - self.viewport_height / 2.0 / self.zoom;
        
        // Disable following
        self.is_following = false;
        
        // Ensure camera stays within bounds
        self.constrain_camera();
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

    /// Get the macroquad Camera2D for rendering
    pub fn get_macroquad_camera(&self) -> Camera2D {
        Camera2D {
            zoom: vec2(
                2.0 / self.viewport_width * self.zoom,
                2.0 / self.viewport_height * self.zoom,
            ),
            target: vec2(
                self.position.x + self.viewport_width / (2.0 * self.zoom),
                self.position.y + self.viewport_height / (2.0 * self.zoom),
            ),
            ..Default::default()
        }
    }
}
