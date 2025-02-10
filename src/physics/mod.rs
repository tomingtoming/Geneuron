use nalgebra as na;

#[derive(Clone)]
pub struct PhysicsState {
    pub position: na::Point2<f32>,
    pub velocity: na::Vector2<f32>,
    pub rotation: f32,
    pub energy: f32,
    rotation_momentum: f32,  // Add rotation momentum for smoother turns
}

impl PhysicsState {
    pub fn new(position: na::Point2<f32>, velocity: na::Vector2<f32>, rotation: f32, energy: f32) -> Self {
        PhysicsState {
            position,
            velocity,
            rotation,
            energy,
            rotation_momentum: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, bounds: (f32, f32)) {
        // Apply momentum to rotation
        self.rotation += self.rotation_momentum * dt;
        
        // Decay rotation momentum
        self.rotation_momentum *= 0.95;
        
        // Normalize rotation to [0, 2π]
        self.rotation = self.rotation.rem_euclid(2.0 * std::f32::consts::PI);
        
        // Update position with current velocity
        let new_pos = self.position + self.velocity * dt;
        
        // Improved boundary handling with smooth bounce
        if new_pos.x < 0.0 || new_pos.x > bounds.0 {
            self.velocity.x *= -0.8;
            // Add slight rotation on bounce
            self.rotation_momentum += if self.velocity.x > 0.0 { -0.2 } else { 0.2 };
            self.position.x = new_pos.x.clamp(0.0, bounds.0);
        } else {
            self.position.x = new_pos.x;
        }
        
        if new_pos.y < 0.0 || new_pos.y > bounds.1 {
            self.velocity.y *= -0.8;
            // Add slight rotation on bounce
            self.rotation_momentum += if self.velocity.y > 0.0 { 0.2 } else { -0.2 };
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
            0.95  // Very sluggish when low energy
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
        
        if current_speed > max_speed {  // 括弧を削除
            self.velocity = new_velocity * (max_speed / current_speed);
        } else {
            self.velocity = new_velocity;
        }

        // Apply rotation force to momentum instead of directly to rotation
        let max_rotation_momentum = if energy_level < 0.3 {
            2.0  // Low energy, low max rotation
        } else if energy_level > 1.0 {
            6.0  // High energy, high max rotation
        } else {
            4.0  // Normal max rotation
        };

        self.rotation_momentum += rotation_force;
        self.rotation_momentum = self.rotation_momentum.clamp(-max_rotation_momentum, max_rotation_momentum);
    }

    pub fn calculate_energy_cost(&self, dt: f32) -> f32 {
        let speed = self.velocity.norm();
        let rotation_cost = self.rotation_momentum.abs() * 0.001;  // Small cost for rotation
        
        // Progressive energy cost based on speed
        let speed_cost = if speed < 10.0 {
            speed * 0.00001  // Very efficient at low speeds
        } else if speed < 50.0 {
            0.0001 * speed  // Linear cost at medium speeds
        } else {
            0.0002 * speed  // Quadratic cost at high speeds
        };
        
        // Base metabolism plus movement costs
        0.005 * dt +  // Reduced base metabolism
        speed_cost * dt +  // Movement cost
        rotation_cost * dt  // Rotation cost
    }

    pub fn distance_to(&self, other: &na::Point2<f32>) -> f32 {
        na::distance(&self.position, other)
    }

    pub fn direction_to(&self, target: &na::Point2<f32>) -> (f32, f32) {
        let direction = target - self.position;
        let distance = direction.norm();
        let target_angle = direction.y.atan2(direction.x);
        let mut angle_diff = (target_angle - self.rotation).rem_euclid(2.0 * std::f32::consts::PI);
        
        // Normalize to [-PI, PI] for shortest turn
        if angle_diff > std::f32::consts::PI {
            angle_diff -= 2.0 * std::f32::consts::PI;
        }
        
        (distance, angle_diff)
    }
}