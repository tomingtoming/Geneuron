use ggez::graphics::Color;
use nalgebra as na;
use rand::Rng;
use std::f32::consts::PI;

use crate::neural::Neural;
use crate::physics::PhysicsState;

#[derive(Clone, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Clone)]
pub struct Creature {
    pub physics: PhysicsState,
    brain: Box<dyn Neural>,
    pub genome: Vec<f32>,
    pub color: Color,
    pub age: f32,
    pub fitness: f32,
    pub gender: Gender,
    pub reproduction_cooldown: f32,
    pub mode_color: Color,
}

impl Creature {
    pub fn new(brain: Box<dyn Neural>) -> Self {
        let mut rng = rand::thread_rng();
        let genome = brain.extract_genome();
        
        Creature {
            physics: PhysicsState::new(
                na::Point2::new(rng.gen_range(0.0..800.0), rng.gen_range(0.0..600.0)),
                na::Vector2::new(0.0, 0.0),
                rng.gen_range(0.0..2.0 * PI),
                1.0,
            ),
            brain,
            genome,
            color: Color::new(rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), 1.0),
            age: 0.0,
            fitness: 0.0,
            gender: if rng.gen_bool(0.5) { Gender::Male } else { Gender::Female },
            reproduction_cooldown: 0.0,
            mode_color: Color::WHITE,
        }
    }

    pub fn update(&mut self, nearby_food: &[na::Point2<f32>], nearby_creatures: &[(usize, na::Point2<f32>, Gender, f32, f32)], dt: f32, bounds: (f32, f32)) {
        self.think(nearby_food, nearby_creatures);
        self.physics.update(dt, bounds);
        self.age += dt;
    }

    fn think(&mut self, nearby_food: &[na::Point2<f32>], nearby_creatures: &[(usize, na::Point2<f32>, Gender, f32, f32)]) {
        let mut inputs = Vec::with_capacity(9);  // Increased input size for new features
        
        // Basic state inputs
        inputs.push(self.physics.energy);
        inputs.push(self.physics.velocity.norm() / 150.0);
        inputs.push((self.physics.rotation / (2.0 * PI)).rem_euclid(1.0));
        
        // Detect nearest same-species creature (flock behavior)
        let nearest_same_species = nearby_creatures.iter()
            .filter(|(_, _, gender, _, _)| *gender == self.gender)  // Only consider same gender
            .map(|(_, pos, ..)| (pos, self.physics.distance_to(pos)))
            .min_by(|(_, dist_a), (_, dist_b)| dist_a.partial_cmp(dist_b).unwrap());
        
        // Add flock behavior inputs
        if let Some((pos, distance)) = nearest_same_species {
            let normalized_distance = distance / 800.0;
            let (_, angle_diff) = self.physics.direction_to(pos);
            inputs.push(normalized_distance);
            inputs.push(angle_diff / PI);
        } else {
            inputs.push(1.0);
            inputs.push(0.0);
        }

        // Food detection with improved priority system
        let food_priority = if self.physics.energy < 0.3 {
            3.0  // Critical priority when very hungry
        } else if self.physics.energy < 0.5 {
            2.0  // High priority when hungry
        } else if self.physics.energy < 0.7 {
            1.2  // Slightly elevated priority
        } else {
            1.0  // Normal priority
        };

        if let Some(nearest) = self.find_nearest_food(nearby_food) {
            let (distance, angle_diff) = self.physics.direction_to(&nearest);
            let normalized_distance = (distance / 800.0) / food_priority;
            inputs.push(normalized_distance);
            inputs.push(angle_diff / PI);
        } else {
            inputs.push(1.0);
            inputs.push(0.0);
        }

        // Mate detection with improved conditions
        let reproduction_priority = if self.physics.energy >= 0.9 {
            2.0  // High priority when energy is abundant
        } else if self.physics.energy >= 0.7 {
            1.5  // Medium priority when energy is good
        } else {
            0.5  // Low priority when energy is not optimal
        };

        let nearest_mate = nearby_creatures.iter()
            .filter(|other| self.can_reproduce_with(other))
            .map(|(_, pos, ..)| (pos, self.physics.distance_to(pos)))
            .min_by(|(_, dist_a), (_, dist_b)| dist_a.partial_cmp(dist_b).unwrap());
        
        if let Some((mate_pos, distance)) = nearest_mate {
            let normalized_distance = (distance / 800.0) / reproduction_priority;
            let (_, angle_diff) = self.physics.direction_to(mate_pos);
            inputs.push(normalized_distance);
            inputs.push(angle_diff / PI);
        } else {
            inputs.push(1.0);
            inputs.push(0.0);
        }
        
        // Neural network processing and movement control
        let outputs = self.brain.process(&inputs);
        
        // Enhanced speed control based on situation
        let base_speed = outputs[0].clamp(0.0, 1.0) * 100.0;
        let forward_speed = match self.physics.energy {
            e if e < 0.2 => base_speed * 0.2,  // Critical energy conservation
            e if e < 0.3 => base_speed * 0.4,  // Heavy energy conservation
            e if e < 0.5 => base_speed * 0.7,  // Moderate energy conservation
            e if e > 1.2 => {
                if nearest_mate.is_some() {
                    base_speed * 1.3  // Extra boost when pursuing mate
                } else {
                    base_speed * 1.1  // Normal boost
                }
            }
            _ => base_speed,
        };

        // Group behavior influence
        let speed_modifier = if let Some((_, distance)) = nearest_same_species {
            if distance < 50.0 {
                0.8  // Slow down when very close to others
            } else if distance < 100.0 {
                0.9  // Slightly slow when moderately close
            } else {
                1.0  // Normal speed otherwise
            }
        } else {
            1.0
        };
        
        let adjusted_speed = forward_speed * speed_modifier;
        
        // Improved rotation control
        let desired_rotation = outputs[1].clamp(0.0, 1.0) * 2.0 * PI;
        let mut angle_diff = desired_rotation - self.physics.rotation;
        
        // Normalize angle to [-PI, PI]
        while angle_diff > PI {
            angle_diff -= 2.0 * PI;
        }
        while angle_diff < -PI {
            angle_diff += 2.0 * PI;
        }
        
        // Dynamic rotation speed based on multiple factors
        let speed_factor = (self.physics.velocity.norm() / 100.0).clamp(0.0, 1.0);
        let energy_factor = (self.physics.energy / 1.5).clamp(0.0, 1.0);
        
        let max_rotation_speed = (1.0 + energy_factor) * (1.0 - speed_factor * 0.6);
        
        // Smooth rotation with situation awareness
        let situation_factor = if nearest_mate.is_some() && self.physics.energy >= 0.7 {
            1.2  // Quicker turning when pursuing mate
        } else if self.physics.energy < 0.3 {
            0.7  // Slower turning when low on energy
        } else {
            1.0
        };
        
        let rotation_speed = angle_diff.signum() * angle_diff.abs().min(max_rotation_speed * 0.1) * situation_factor;
        
        // Calculate movement force with improved directional control
        let force = na::Vector2::new(
            adjusted_speed * self.physics.rotation.cos(),
            adjusted_speed * self.physics.rotation.sin()
        );
        
        // Apply final movement updates
        self.physics.apply_force(force, rotation_speed, 0.1, self.physics.energy);
        
        // Update mode color with more detailed state indication and smoother transitions
        self.mode_color = match (self.physics.energy, &nearest_mate, &nearest_same_species) {
            (energy, Some(_), _) if energy >= 0.7 => {
                Color::new(1.0, 0.0, 0.0, 1.0)  // Bright red for reproduction mode
            },
            (energy, _, _) if energy < 0.3 => {
                Color::new(0.0, 0.0, 1.0, 1.0)  // Deep blue for very hungry
            },
            (energy, _, _) if energy < 0.5 => {
                Color::new(0.3, 0.3, 1.0, 1.0)  // Lighter blue for somewhat hungry
            },
            (_, _, Some((_, ref distance))) if *distance < 50.0 => {
                Color::new(0.0, 1.0, 0.0, 1.0)  // Green for close group behavior
            },
            (_, _, Some((_, ref distance))) if *distance < 100.0 => {
                Color::new(0.5, 1.0, 0.5, 1.0)  // Light green for moderate group behavior
            },
            _ => {
                Color::new(0.7, 0.7, 0.7, 1.0)  // Gray for solo exploration
            }
        };
    }

    pub fn can_reproduce_with(&self, other: &(usize, na::Point2<f32>, Gender, f32, f32)) -> bool {
        let (_, pos, gender, cooldown, energy) = other;
        *gender != self.gender &&
        *cooldown <= 0.0 &&
        *energy >= 0.7 &&
        self.reproduction_cooldown <= 0.0 &&
        self.physics.energy >= 0.7 &&
        self.physics.distance_to(pos) < 30.0
    }

    pub fn reproduce_with(&self, other: &Creature) -> Creature {
        let mut child = Creature::new(self.brain.clone());
        let mut rng = rand::thread_rng();

        // Crossover using genomes
        let crossover_point = rng.gen_range(0..self.genome.len());
        child.genome = self.genome[..crossover_point].to_vec();
        child.genome.extend_from_slice(&other.genome[crossover_point..]);

        // Apply genome to brain
        child.brain.apply_genome(&child.genome);

        // Inherit color
        child.color = Color::new(
            ((self.color.r + other.color.r) * 0.5 + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
            ((self.color.g + other.color.g) * 0.5 + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
            ((self.color.b + other.color.b) * 0.5 + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
            1.0
        );

        // Mutate
        self.mutate(&mut child, 0.1);

        // Set position near parents
        child.physics.position = na::Point2::new(
            (self.physics.position.x + other.physics.position.x) * 0.5 + rng.gen_range(-20.0..20.0),
            (self.physics.position.y + other.physics.position.y) * 0.5 + rng.gen_range(-20.0..20.0)
        );

        child
    }

    fn mutate(&self, child: &mut Creature, mutation_rate: f32) {
        let mut rng = rand::thread_rng();
        
        // Mutate brain
        child.brain.mutate(mutation_rate);
        
        // Update genome from mutated brain
        child.genome = child.brain.extract_genome();
        
        // Mutate color
        if rng.gen::<f32>() < mutation_rate {
            child.color = Color::new(
                (child.color.r + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
                (child.color.g + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
                (child.color.b + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
                1.0,
            );
        }
    }

    fn find_nearest_food(&self, food_sources: &[na::Point2<f32>]) -> Option<na::Point2<f32>> {
        food_sources.iter()
            .min_by(|a, b| {
                let dist_a = self.physics.distance_to(a);
                let dist_b = self.physics.distance_to(b);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .copied()
    }
}