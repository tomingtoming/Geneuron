use nalgebra as na;

#[derive(Clone)]
pub struct PhysicsState {
    pub position: na::Point2<f32>,
    pub velocity: na::Vector2<f32>,
    pub rotation: f32,
    pub energy: f32,
    rotation_momentum: f32, // Add rotation momentum for smoother turns
}

impl PhysicsState {
    pub fn new(
        position: na::Point2<f32>,
        velocity: na::Vector2<f32>,
        rotation: f32,
        energy: f32,
    ) -> Self {
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

        // トーラス構造の処理（境界を超えた時にワープ）
        if new_pos.x < 0.0 {
            self.position.x = new_pos.x + bounds.0;
        } else if new_pos.x > bounds.0 {
            self.position.x = new_pos.x - bounds.0;
        } else {
            self.position.x = new_pos.x;
        }

        if new_pos.y < 0.0 {
            self.position.y = new_pos.y + bounds.1;
        } else if new_pos.y > bounds.1 {
            self.position.y = new_pos.y - bounds.1;
        } else {
            self.position.y = new_pos.y;
        }
    }

    pub fn apply_force(
        &mut self,
        force: na::Vector2<f32>,
        rotation_force: f32,
        _dt: f32,
        energy_level: f32,
    ) {
        // Base responsiveness on energy level
        let base_inertia = if energy_level > 1.0 {
            0.85 // More responsive when energy is high
        } else if energy_level < 0.3 {
            0.95 // Very sluggish when low energy
        } else {
            0.9 // Normal responsiveness
        };

        // Calculate max speed based on energy level
        let max_speed = if energy_level > 1.0 {
            220.0 // 110から220に増加
        } else if energy_level < 0.3 {
            60.0 // 30から60に増加
        } else {
            200.0 // 100から200に増加
        };

        // Update velocity with inertia and speed limit
        let new_velocity = self.velocity * base_inertia + force * (1.0 - base_inertia);
        let current_speed = new_velocity.norm();

        if current_speed > max_speed {
            // 括弧を削除
            self.velocity = new_velocity * (max_speed / current_speed);
        } else {
            self.velocity = new_velocity;
        }

        // Apply rotation force to momentum instead of directly to rotation
        let max_rotation_momentum = if energy_level < 0.3 {
            2.0 // Low energy, low max rotation
        } else if energy_level > 1.0 {
            6.0 // High energy, high max rotation
        } else {
            4.0 // Normal max rotation
        };

        self.rotation_momentum += rotation_force;
        self.rotation_momentum = self
            .rotation_momentum
            .clamp(-max_rotation_momentum, max_rotation_momentum);
    }

    pub fn calculate_energy_cost(&self, dt: f32) -> f32 {
        let speed = self.velocity.norm();
        let rotation_cost = self.rotation_momentum.abs() * 0.001; // Small cost for rotation

        // Progressive energy cost based on speed
        let speed_cost = if speed < 20.0 {
            // 10から20に増加
            speed * 0.00001 // Very efficient at low speeds
        } else if speed < 100.0 {
            // 50から100に増加
            0.0001 * speed // Linear cost at medium speeds
        } else {
            0.0002 * speed // Quadratic cost at high speeds
        };

        // Base metabolism plus movement costs
        0.003 * dt +  // 0.005から0.003に減少（広い世界での長期生存を可能に）
        speed_cost * dt +  // Movement cost
        rotation_cost * dt // Rotation cost
    }

    pub fn distance_to(&self, other: &na::Point2<f32>, bounds: (f32, f32)) -> f32 {
        // トーラス構造を考慮した最短距離の計算
        let mut dx = (other.x - self.position.x).abs();
        let mut dy = (other.y - self.position.y).abs();

        // X軸方向の最短距離
        if dx > bounds.0 / 2.0 {
            dx = bounds.0 - dx;
        }

        // Y軸方向の最短距離
        if dy > bounds.1 / 2.0 {
            dy = bounds.1 - dy;
        }

        (dx * dx + dy * dy).sqrt()
    }

    pub fn direction_to(&self, target: &na::Point2<f32>, bounds: (f32, f32)) -> (f32, f32) {
        // トーラス構造を考慮した最短距離と方向の計算
        let mut dx = target.x - self.position.x;
        let mut dy = target.y - self.position.y;

        // X軸方向の最短距離を計算
        if dx.abs() > bounds.0 / 2.0 {
            if dx > 0.0 {
                dx = dx - bounds.0;
            } else {
                dx = dx + bounds.0;
            }
        }

        // Y軸方向の最短距離を計算
        if dy.abs() > bounds.1 / 2.0 {
            if dy > 0.0 {
                dy = dy - bounds.1;
            } else {
                dy = dy + bounds.1;
            }
        }

        let direction = na::Vector2::new(dx, dy);
        let distance = direction.norm();

        if distance == 0.0 {
            return (0.0, 0.0);
        }

        let target_angle = dy.atan2(dx);
        let mut angle_diff = (target_angle - self.rotation).rem_euclid(2.0 * std::f32::consts::PI);

        // Normalize to [-PI, PI] for shortest turn
        if angle_diff > std::f32::consts::PI {
            angle_diff -= 2.0 * std::f32::consts::PI;
        }

        (distance, angle_diff)
    }
}
