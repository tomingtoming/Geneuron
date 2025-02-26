#[cfg(test)]
mod camera_tests {
    use crate::camera::Camera;
    use crate::creature::Creature;
    use macroquad::prelude::Vec2;
    use nalgebra as na;
    use crate::world::World;

    const WORLD_WIDTH: f32 = 1000.0;
    const WORLD_HEIGHT: f32 = 800.0;
    const SCREEN_WIDTH: f32 = 800.0;
    const SCREEN_HEIGHT: f32 = 600.0;
    
    fn setup() -> (Camera, World) {
        let world = World::new(WORLD_WIDTH, WORLD_HEIGHT);
        let camera = Camera::new(SCREEN_WIDTH, SCREEN_HEIGHT, (WORLD_WIDTH, WORLD_HEIGHT));
        (camera, world)
    }
    
    #[test]
    fn test_camera_constraints() {
        let (mut camera, world) = setup();
        
        // Test camera position constraints
        camera.position = na::Point2::new(-500.0, -400.0);
        camera.constrain_camera();
        
        // Camera should be constrained to valid boundaries based on zoom
        let min_x = camera.viewport_width / (2.0 * camera.zoom);
        let min_y = camera.viewport_height / (2.0 * camera.zoom);
        let max_x = world.world_bounds.0 - (camera.viewport_width / (2.0 * camera.zoom));
        let max_y = world.world_bounds.1 - (camera.viewport_height / (2.0 * camera.zoom));
        
        assert!(camera.position.x >= -min_x && camera.position.x <= max_x);
        assert!(camera.position.y >= -min_y && camera.position.y <= max_y);
    }
    
    #[test]
    fn test_camera_reset() {
        let (mut camera, _world) = setup();
        
        // Move camera to a different position and zoom
        camera.position = na::Point2::new(300.0, 400.0);
        camera.zoom = 0.5;
        
        // Store the current position
        let initial_position = camera.position.clone();
        let initial_zoom = camera.zoom;
        
        // Reset the camera
        camera.reset_view();
        
        // Verify that target position has changed
        assert!(camera.target_position.x != initial_position.x,
                "Target X position should change after reset");
        assert!(camera.target_position.y != initial_position.y, 
                "Target Y position should change after reset");
        
        // Verify target zoom has changed
        assert!(camera.target_zoom != initial_zoom,
                "Target zoom should change after reset");
        
        // Get the actual target zoom from reset_view() implementation
        let width_ratio = SCREEN_WIDTH / WORLD_WIDTH;
        let height_ratio = SCREEN_HEIGHT / WORLD_HEIGHT;
        let expected_zoom = (width_ratio.min(height_ratio) * 0.8).max(0.33);
        assert_eq!(camera.target_zoom, expected_zoom);
        
        // Run a few updates to see that position moves toward target
        let init_diff_x = (camera.position.x - camera.target_position.x).abs();
        let init_diff_y = (camera.position.y - camera.target_position.y).abs();
        
        // Run updates
        for _ in 0..5 {
            camera.update(0.1);
        }
        
        // After updates, position should have moved closer to target
        let new_diff_x = (camera.position.x - camera.target_position.x).abs();
        let new_diff_y = (camera.position.y - camera.target_position.y).abs();
        
        assert!(new_diff_x < init_diff_x || new_diff_x < 0.1,
                "Camera should move closer to target X: {} -> {}", init_diff_x, new_diff_x);
        assert!(new_diff_y < init_diff_y || new_diff_y < 0.1,
                "Camera should move closer to target Y: {} -> {}", init_diff_y, new_diff_y);
    }
    
    #[test]
    fn test_camera_follow_mode() {
        let (mut camera, mut world) = setup();
        
        // Create a creature at a specific position
        let position = na::Point2::new(300.0, 400.0);
        let creature = Creature::new(position);
        
        // Add creature to world and get its index
        let _creature_index = world.creatures.len();
        world.creatures.push(creature);
        
        // Record initial camera position
        let initial_pos = camera.position.clone();
        
        // Set camera to follow mode and set follow target
        camera.set_follow_target(Some(position));
        camera.set_following(true);
        
        // Apply updates to camera
        for _ in 0..10 {
            camera.update(0.1);
        }
        
        // After updates, the camera should have moved from its initial position
        // toward the follow target (we don't test exact positioning)
        assert!(camera.position != initial_pos,
                "Camera position should change when following");
        
        // Test that follow mode can be toggled
        assert_eq!(camera.is_following(), true);
        camera.set_following(false);
        assert_eq!(camera.is_following(), false);
    }
    
    #[test]
    fn test_camera_world_wrapping() {
        let (mut camera, world) = setup();
        
        // Test camera movement near world edges
        // Position camera near the right edge
        camera.position = na::Point2::new(world.world_bounds.0 - 50.0, world.world_bounds.1 / 2.0);
        
        // Test that world wrapping is calculated correctly
        let screen_pos = camera.world_to_screen(na::Point2::new(50.0, world.world_bounds.1 / 2.0));
        let wrapped_pos = camera.world_to_screen(na::Point2::new(world.world_bounds.0 + 50.0, world.world_bounds.1 / 2.0));
        
        // Position should be far apart since we're not accounting for wrapping in the test
        // (The actual wrapping happens in the rendering code, not in world_to_screen)
        assert!((screen_pos.x - wrapped_pos.x).abs() > 1.0);
    }
    
    #[test]
    fn test_camera_zoom() {
        let (mut camera, _world) = setup();
        
        // Save original position
        let original_position = camera.position.clone();
        let _original_target = camera.target_position.clone();
        
        // Get screen center
        let screen_center = Vec2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);
        
        // Test zooming through the handle_mouse_wheel_zoom method
        camera.handle_mouse_wheel_zoom(0.5, Some(screen_center));
        
        // Target zoom should increase
        assert!(camera.target_zoom > camera.zoom);
        
        // Position and target position should remain unchanged initially
        assert_eq!(camera.position.x, original_position.x);
        assert_eq!(camera.position.y, original_position.y);
        
        // Update camera to apply zoom changes
        for _ in 0..10 {
            camera.update(0.1);
        }
        
        // After updates, zoom should have changed
        assert!(camera.zoom > 0.33);
        
        // The camera center should stay relatively stable 
        let view_width_before = SCREEN_WIDTH / camera.zoom;
        let view_height_before = SCREEN_HEIGHT / camera.zoom;
        
        let center_before = na::Point2::new(
            camera.position.x + view_width_before / 2.0,
            camera.position.y + view_height_before / 2.0
        );
        
        // Zoom out
        camera.handle_mouse_wheel_zoom(-0.5, Some(screen_center));
        
        // Update camera to apply zoom changes
        for _ in 0..10 {
            camera.update(0.1);
        }
        
        let view_width_after = SCREEN_WIDTH / camera.zoom;
        let view_height_after = SCREEN_HEIGHT / camera.zoom;
        
        let center_after = na::Point2::new(
            camera.position.x + view_width_after / 2.0,
            camera.position.y + view_height_after / 2.0
        );
        
        // Check that the center of view is relatively stable (may drift slightly due to rounding)
        assert!((center_before.x - center_after.x).abs() < 5.0);
        assert!((center_before.y - center_after.y).abs() < 5.0);
    }
}
