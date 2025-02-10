use ggez::graphics::Color;
use nalgebra as na;
use rand::Rng;
use std::f32::consts::PI;

use crate::neural::Neural;
use crate::physics::PhysicsState;

#[derive(Clone, PartialEq, Debug)] // Add Debug trait
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
    pub behavior_state: BehaviorState,
    state_transition_timer: f32,
}

#[derive(Clone, Debug)] // Add Debug trait
pub enum BehaviorState {
    Exploring,
    Feeding,
    Socializing,
    Reproducing,
    Resting,
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
            color: Color::new(
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                1.0,
            ),
            age: 0.0,
            fitness: 0.0,
            gender: if rng.gen_bool(0.5) {
                Gender::Male
            } else {
                Gender::Female
            },
            reproduction_cooldown: 0.0,
            mode_color: Color::WHITE,
            behavior_state: BehaviorState::Exploring,
            state_transition_timer: 0.0,
        }
    }

    pub fn update(
        &mut self,
        nearby_food: &[na::Point2<f32>],
        nearby_creatures: &[(usize, na::Point2<f32>, Gender, f32, f32)],
        dt: f32,
        bounds: (f32, f32),
    ) {
        // Update state transition timer
        self.state_transition_timer -= dt;
        if self.state_transition_timer <= 0.0 {
            self.update_behavior_state(nearby_food, nearby_creatures, bounds);
            self.state_transition_timer = 0.5; // Check state every 0.5 seconds
        }

        self.think(nearby_food, nearby_creatures, bounds);
        self.physics.update(dt, bounds);
        self.age += dt;
    }

    fn update_behavior_state(
        &mut self,
        nearby_food: &[na::Point2<f32>],
        nearby_creatures: &[(usize, na::Point2<f32>, Gender, f32, f32)],
        bounds: (f32, f32),
    ) {
        let has_nearby_food = nearby_food
            .iter()
            .any(|food| self.physics.distance_to(food, bounds) < 200.0); // 検出範囲を100から200に増加

        let has_potential_mate = nearby_creatures
            .iter()
            .any(|other| self.can_reproduce_with(other, bounds));

        let has_nearby_friends = nearby_creatures
            .iter()
            .filter(
                |(_, pos, gender, _, _)| {
                    *gender == self.gender && self.physics.distance_to(pos, bounds) < 100.0
                }, // 検出範囲を50から100に増加
            )
            .count()
            >= 2;

        self.behavior_state = match (
            self.physics.energy,
            has_nearby_food,
            has_potential_mate,
            has_nearby_friends,
        ) {
            (energy, _, _, _) if energy < 0.3 => BehaviorState::Feeding,
            (energy, _, true, _) if energy >= 0.7 => BehaviorState::Reproducing,
            (energy, _, _, true) if energy < 0.5 => BehaviorState::Resting,
            (_, true, _, _) => BehaviorState::Feeding,
            (_, _, _, true) => BehaviorState::Socializing,
            _ => BehaviorState::Exploring,
        };
    }

    fn think(
        &mut self,
        nearby_food: &[na::Point2<f32>],
        nearby_creatures: &[(usize, na::Point2<f32>, Gender, f32, f32)],
        bounds: (f32, f32),
    ) {
        let mut inputs = Vec::with_capacity(9);

        // Basic state inputs
        inputs.push(self.physics.energy);
        inputs.push(self.physics.velocity.norm() / 150.0);
        inputs.push((self.physics.rotation / (2.0 * PI)).rem_euclid(1.0));

        // Dynamic weights based on current state
        let (food_weight, mate_weight, social_weight) = match self.behavior_state {
            BehaviorState::Feeding => (2.0, 0.5, 0.5),
            BehaviorState::Reproducing => (0.5, 2.0, 0.5),
            BehaviorState::Socializing => (0.5, 0.5, 2.0),
            BehaviorState::Resting => (0.3, 0.3, 1.0),
            BehaviorState::Exploring => (1.0, 1.0, 1.0),
        };

        // Social influence (flock behavior)
        let nearest_friend = nearby_creatures
            .iter()
            .filter(|(_, _, gender, _, _)| *gender == self.gender)
            .map(|(_, pos, ..)| (pos, self.physics.distance_to(pos, bounds)))
            .min_by(|(_, dist_a), (_, dist_b)| dist_a.partial_cmp(dist_b).unwrap());

        if let Some((pos, distance)) = nearest_friend {
            let normalized_distance = (distance / 1600.0) / social_weight; // 800から1600に増加
            let (_, angle_diff) = self.physics.direction_to(pos, bounds);
            inputs.push(normalized_distance);
            inputs.push(angle_diff / PI);
        } else {
            inputs.push(1.0);
            inputs.push(0.0);
        }

        // Food seeking with dynamic priority
        if let Some(nearest) = self.find_nearest_food(nearby_food, bounds) {
            let (distance, angle_diff) = self.physics.direction_to(&nearest, bounds);
            let normalized_distance = (distance / 1600.0) / food_weight; // 800から1600に増加
            inputs.push(normalized_distance);
            inputs.push(angle_diff / PI);
        } else {
            inputs.push(1.0);
            inputs.push(0.0);
        }

        // Mate seeking with dynamic priority
        if let Some((mate_pos, distance)) = nearby_creatures
            .iter()
            .filter(|other| self.can_reproduce_with(other, bounds))
            .map(|(_, pos, ..)| (pos, self.physics.distance_to(pos, bounds)))
            .min_by(|(_, dist_a), (_, dist_b)| dist_a.partial_cmp(dist_b).unwrap())
        {
            let normalized_distance = (distance / 1600.0) / mate_weight; // 800から1600に増加
            let (_, angle_diff) = self.physics.direction_to(mate_pos, bounds);
            inputs.push(normalized_distance);
            inputs.push(angle_diff / PI);
        } else {
            inputs.push(1.0);
            inputs.push(0.0);
        }

        // Process neural network
        let outputs = self.brain.process(&inputs);

        // Apply movement based on state and energy
        let base_speed = outputs[0].clamp(0.0, 1.0)
            * match self.behavior_state {
                BehaviorState::Resting => 30.0,
                BehaviorState::Feeding => 80.0,
                BehaviorState::Reproducing => 100.0,
                BehaviorState::Socializing => 60.0,
                BehaviorState::Exploring => 70.0,
            };

        let speed_modifier = match self.physics.energy {
            e if e < 0.2 => 0.3,
            e if e < 0.3 => 0.5,
            e if e < 0.5 => 0.8,
            e if e > 1.2 => 1.2,
            _ => 1.0,
        };

        let final_speed = base_speed * speed_modifier;

        // Calculate movement direction
        let force = na::Vector2::new(
            final_speed * self.physics.rotation.cos(),
            final_speed * self.physics.rotation.sin(),
        );

        // Smooth rotation based on state
        let rotation_modifier = match self.behavior_state {
            BehaviorState::Resting => 0.5,
            BehaviorState::Feeding => 1.2,
            BehaviorState::Reproducing => 1.5,
            _ => 1.0,
        };

        let rotation_force = (outputs[1] - 0.5) * 2.0 * PI * rotation_modifier;

        // Apply movement
        self.physics
            .apply_force(force, rotation_force, 0.1, self.physics.energy);

        // Update color based on state and energy
        self.mode_color = match self.behavior_state {
            BehaviorState::Reproducing => Color::RED,
            BehaviorState::Feeding => Color::BLUE,
            BehaviorState::Socializing => Color::GREEN,
            BehaviorState::Resting => Color::new(0.5, 0.5, 0.5, 1.0),
            BehaviorState::Exploring => Color::WHITE,
        };
    }

    pub fn can_reproduce_with(
        &self,
        other: &(usize, na::Point2<f32>, Gender, f32, f32),
        bounds: (f32, f32),
    ) -> bool {
        let (_, pos, gender, cooldown, energy) = other;
        *gender != self.gender
            && *cooldown <= 0.0
            && *energy >= 0.7
            && self.reproduction_cooldown <= 0.0
            && self.physics.energy >= 0.7
            && self.physics.distance_to(pos, bounds) < 60.0 // 30から60に増加
    }

    fn find_nearest_food(
        &self,
        food_sources: &[na::Point2<f32>],
        bounds: (f32, f32),
    ) -> Option<na::Point2<f32>> {
        food_sources
            .iter()
            .min_by(|a, b| {
                let dist_a = self.physics.distance_to(a, bounds);
                let dist_b = self.physics.distance_to(b, bounds);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .copied()
    }

    pub fn reproduce_with(&self, other: &Creature) -> Creature {
        let mut child = Creature::new(self.brain.clone());
        let mut rng = rand::thread_rng();

        // Crossover using genomes
        let crossover_point = rng.gen_range(0..self.genome.len());
        child.genome = self.genome[..crossover_point].to_vec();
        child
            .genome
            .extend_from_slice(&other.genome[crossover_point..]);

        // Apply genome to brain
        child.brain.apply_genome(&child.genome);

        // Inherit color
        child.color = Color::new(
            ((self.color.r + other.color.r) * 0.5 + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
            ((self.color.g + other.color.g) * 0.5 + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
            ((self.color.b + other.color.b) * 0.5 + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
            1.0,
        );

        // Mutate
        self.mutate(&mut child, 0.1);

        // Set position near parents
        child.physics.position = na::Point2::new(
            (self.physics.position.x + other.physics.position.x) * 0.5 + rng.gen_range(-20.0..20.0),
            (self.physics.position.y + other.physics.position.y) * 0.5 + rng.gen_range(-20.0..20.0),
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
}
