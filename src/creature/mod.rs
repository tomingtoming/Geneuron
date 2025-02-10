use ::rand::prelude::*;
use ::rand::thread_rng;
use macroquad::prelude::*;
use nalgebra as na;

use crate::neural::{FeedForwardNetwork, Neural};
use crate::physics::PhysicsBody;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Clone)]
pub struct Creature {
    pub physics: PhysicsBody,
    pub brain: FeedForwardNetwork,
    pub color: Color,
    pub mode_color: Color,
    pub age: f32,
    pub fitness: f32,
    pub behavior_state: BehaviorState,
    pub reproduction_cooldown: f32,
    pub gender: Gender,
}

#[derive(Debug, Clone, Copy)]
pub enum BehaviorState {
    Wandering,
    Seeking,
    Resting,
    Mating,
}

impl Creature {
    pub fn new(position: na::Point2<f32>) -> Self {
        let mut rng = thread_rng();
        let gender = if rng.gen::<bool>() {
            Gender::Male
        } else {
            Gender::Female
        };
        let color = match gender {
            Gender::Male => BLUE,
            Gender::Female => PINK,
        };

        Creature {
            physics: PhysicsBody::new(position),
            brain: FeedForwardNetwork::new(5, 4),
            color,
            mode_color: WHITE,
            age: 0.0,
            fitness: 0.0,
            behavior_state: BehaviorState::Wandering,
            reproduction_cooldown: 0.0,
            gender,
        }
    }

    pub fn reproduce_with(&self, other: &Creature) -> Creature {
        let mut child = Creature::new(self.physics.position);
        child.brain = self.brain.crossover_with(&other.brain);
        child.brain.mutate();
        child
    }

    pub fn can_reproduce_with(
        &self,
        other_data: &(usize, na::Point2<f32>, Gender, f32, f32),
    ) -> bool {
        let (_, _, other_gender, other_energy, _) = *other_data;
        self.reproduction_cooldown <= 0.0
            && self.physics.energy >= 0.7
            && other_gender != self.gender
            && other_energy >= 0.7
    }

    pub fn update(
        &mut self,
        dt: f32,
        nearby_food: &[na::Point2<f32>],
        nearby_creatures: &[(usize, na::Point2<f32>, Gender, f32, f32)],
        bounds: (f32, f32),
    ) {
        self.age += dt;
        self.reproduction_cooldown = (self.reproduction_cooldown - dt).max(0.0);

        // Think and update behavior
        self.think(nearby_food, nearby_creatures, bounds);

        // Apply energy cost based on movement and state
        let speed = self.physics.velocity.norm();
        let base_cost = match self.behavior_state {
            BehaviorState::Resting => 0.001,
            _ => 0.002 + speed * 0.001,
        };
        self.physics.energy -= base_cost * dt;

        // Update position
        self.physics.update(dt, bounds);

        // Update fitness
        self.fitness = self.age * self.physics.energy;
    }

    fn update_behavior_state(&mut self, nearest_food_dist: f32, nearest_mate_dist: Option<f32>) {
        self.behavior_state = if self.physics.energy < 0.3 {
            if nearest_food_dist < 100.0 {
                self.mode_color = YELLOW;
                BehaviorState::Seeking
            } else {
                self.mode_color = RED;
                BehaviorState::Wandering
            }
        } else if self.physics.energy > 0.7 && nearest_mate_dist.is_some() {
            self.mode_color = GREEN;
            BehaviorState::Mating
        } else if self.physics.energy > 0.9 {
            self.mode_color = SKYBLUE;
            BehaviorState::Resting
        } else {
            self.mode_color = WHITE;
            BehaviorState::Wandering
        };
    }

    fn think(
        &mut self,
        nearby_food: &[na::Point2<f32>],
        nearby_creatures: &[(usize, na::Point2<f32>, Gender, f32, f32)],
        bounds: (f32, f32),
    ) {
        // Prepare neural network inputs
        let mut inputs = vec![
            self.physics.energy,
            self.physics.velocity.norm() / 200.0,
            self.physics.rotation / std::f32::consts::PI,
        ];

        // Find nearest food
        let nearest_food = nearby_food.iter().min_by(|a, b| {
            let dist_a = self.physics.distance_to(a, bounds);
            let dist_b = self.physics.distance_to(b, bounds);
            dist_a.partial_cmp(&dist_b).unwrap()
        });

        // Add food inputs
        if let Some(food_pos) = nearest_food {
            let distance = self.physics.distance_to(food_pos, bounds);
            let normalized_distance = distance / 800.0;
            let (_, angle_diff) = self.physics.direction_to(food_pos, bounds);
            inputs.push(normalized_distance);
            inputs.push(angle_diff / std::f32::consts::PI);
        } else {
            inputs.push(1.0);
            inputs.push(0.0);
        }

        // Find nearest potential mate
        let nearest_mate = nearby_creatures
            .iter()
            .filter(|(_, _, gender, energy, _)| *gender != self.gender && *energy >= 0.7)
            .min_by(|a, b| {
                let dist_a = self.physics.distance_to(&a.1, bounds);
                let dist_b = self.physics.distance_to(&b.1, bounds);
                dist_a.partial_cmp(&dist_b).unwrap()
            });

        // Update behavior state
        self.update_behavior_state(
            nearest_food.map_or(f32::INFINITY, |pos| self.physics.distance_to(pos, bounds)),
            nearest_mate.map(|(_, pos, _, _, _)| self.physics.distance_to(pos, bounds)),
        );

        // Process neural network
        let outputs = self.brain.process(&inputs);

        // Apply movement based on state and energy
        let base_speed = outputs[0].clamp(0.0, 1.0)
            * match self.behavior_state {
                BehaviorState::Resting => 30.0,
                _ => 150.0,
            };

        let target_rotation = outputs[1] * std::f32::consts::PI * 2.0;
        self.physics.apply_movement(base_speed, target_rotation);
    }
}
