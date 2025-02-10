use nalgebra as na;
use rand::prelude::*;

#[derive(Clone)]
pub struct PhysicsState {
    pub position: na::Point2<f32>,
    pub velocity: na::Vector2<f32>,
    pub rotation: f32,
    pub energy: f32,
}

impl PhysicsState {
    pub fn new(position: na::Point2<f32>, velocity: na::Vector2<f32>, rotation: f32, energy: f32) -> Self {
        PhysicsState {
            position,
            velocity,
            rotation,
            energy,
        }
    }

    pub fn update(&mut self, dt: f32, bounds: (f32, f32)) {
        // Update position with current velocity
        let new_pos = self.position + self.velocity * dt;
        
        // Boundary handling with smooth bounce
        if new_pos.x < 0.0 || new_pos.x > bounds.0 {
            self.velocity.x *= -0.8;
            self.position.x = new_pos.x.clamp(0.0, bounds.0);
        } else {
            self.position.x = new_pos.x;
        }
        
        if new_pos.y < 0.0 || new_pos.y > bounds.1 {
            self.velocity.y *= -0.8;
            self.position.y = new_pos.y.clamp(0.0, bounds.1);
        } else {
            self.position.y = new_pos.y;
        }
    }

    pub fn apply_force(&mut self, force: na::Vector2<f32>, rotation_force: f32, _dt: f32, energy_level: f32) {
        // Base responsiveness on energy level
        let base_inertia = if energy_level > 1.0 {
            0.85  // More responsive when energy is high
        } else if energy_level < 0.3 {
            0.95  // Very sluggish when energy is low
        } else {
            0.9   // Normal responsiveness
        };

        // Calculate max speed based on energy level
        let max_speed = if energy_level > 1.0 {
            110.0
        } else if energy_level < 0.3 {
            30.0
        } else {
            100.0
        };

        // Update velocity with inertia and speed limit
        let new_velocity = self.velocity * base_inertia + force * (1.0 - base_inertia);
        let current_speed = new_velocity.norm();
        
        if current_speed > max_speed {
            self.velocity = new_velocity * (max_speed / current_speed);
        } else {
            self.velocity = new_velocity;
        }

        // Apply rotation with smooth damping
        let rotation_damping = if energy_level < 0.3 {
            0.95  // Strong damping when low energy
        } else if energy_level < 0.5 {
            0.9   // Moderate damping
        } else {
            0.85  // Normal damping
        };

        // Add slight randomness to prevent perfect rotations
        let mut rng = rand::thread_rng();
        let random_factor = 1.0 + rng.gen_range(-0.01..0.01);
        
        self.rotation += rotation_force * (1.0 - rotation_damping) * random_factor;
        
        // Normalize rotation to [0, 2π]
        self.rotation = self.rotation.rem_euclid(2.0 * std::f32::consts::PI);
    }

    pub fn calculate_energy_cost(&self, dt: f32) -> f32 {
        let speed = self.velocity.norm();
        let rotation_cost = self.rotation.abs() * 0.001;  // Small cost for rotation
        
        // Base metabolism cost plus movement costs
        0.01 * dt +  // Base metabolism
        speed * speed * 0.00005 * dt +  // Movement cost
        rotation_cost * dt  // Rotation cost
    }

    pub fn distance_to(&self, other: &na::Point2<f32>) -> f32 {
        na::distance(&self.position, other)
    }

    pub fn direction_to(&self, target: &na::Point2<f32>) -> (f32, f32) {
        let direction = target - self.position;
        let distance = direction.norm();
        let target_angle = direction.y.atan2(direction.x);
        let angle_diff = (target_angle - self.rotation).rem_euclid(2.0 * std::f32::consts::PI);
        (distance, angle_diff)
    }
}