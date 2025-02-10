use nalgebra as na;

#[derive(Clone)]
pub struct PhysicsBody {
    pub position: na::Point2<f32>,
    pub velocity: na::Vector2<f32>,
    pub rotation: f32,
    pub energy: f32,
}

impl PhysicsBody {
    pub fn new(position: na::Point2<f32>) -> Self {
        PhysicsBody {
            position,
            velocity: na::Vector2::new(0.0, 0.0),
            rotation: 0.0,
            energy: 1.0,
        }
    }

    pub fn update(&mut self, dt: f32, bounds: (f32, f32)) {
        // Update position based on velocity
        self.position += self.velocity * dt;

        // Apply world bounds (torus)
        if self.position.x < 0.0 {
            self.position.x += bounds.0;
        } else if self.position.x > bounds.0 {
            self.position.x -= bounds.0;
        }

        if self.position.y < 0.0 {
            self.position.y += bounds.1;
        } else if self.position.y > bounds.1 {
            self.position.y -= bounds.1;
        }

        // Apply drag
        self.velocity *= 0.95;
    }

    pub fn apply_movement(&mut self, speed: f32, target_rotation: f32) {
        // Smooth rotation
        let rotation_diff = target_rotation - self.rotation;
        let wrapped_diff = if rotation_diff > std::f32::consts::PI {
            rotation_diff - 2.0 * std::f32::consts::PI
        } else if rotation_diff < -std::f32::consts::PI {
            rotation_diff + 2.0 * std::f32::consts::PI
        } else {
            rotation_diff
        };
        self.rotation += wrapped_diff * 0.1;

        // Apply movement in direction of rotation
        self.velocity += na::Vector2::new(
            self.rotation.cos() * speed,
            self.rotation.sin() * speed,
        ) * 0.1;
    }

    pub fn distance_to(&self, other: &na::Point2<f32>, bounds: (f32, f32)) -> f32 {
        let dx = (self.position.x - other.x).abs();
        let dy = (self.position.y - other.y).abs();
        let wrapped_dx = dx.min(bounds.0 - dx);
        let wrapped_dy = dy.min(bounds.1 - dy);
        (wrapped_dx * wrapped_dx + wrapped_dy * wrapped_dy).sqrt()
    }

    pub fn direction_to(&self, other: &na::Point2<f32>, bounds: (f32, f32)) -> (f32, f32) {
        let dx = other.x - self.position.x;
        let dy = other.y - self.position.y;
        
        let wrapped_dx = if dx.abs() > bounds.0 / 2.0 {
            if dx > 0.0 {
                dx - bounds.0
            } else {
                dx + bounds.0
            }
        } else {
            dx
        };

        let wrapped_dy = if dy.abs() > bounds.1 / 2.0 {
            if dy > 0.0 {
                dy - bounds.1
            } else {
                dy + bounds.1
            }
        } else {
            dy
        };

        let distance = (wrapped_dx * wrapped_dx + wrapped_dy * wrapped_dy).sqrt();
        let angle = wrapped_dy.atan2(wrapped_dx);
        (distance, angle)
    }
}
