use ::rand::prelude::*;
use ::rand::thread_rng;
use nalgebra as na;

use crate::creature::{Creature, Gender};
use crate::food::FoodManager;

#[derive(Clone)]
pub struct World {
    pub creatures: Vec<Creature>,
    pub food_manager: FoodManager,
    pub world_bounds: (f32, f32),
    pub generation: usize,
    pub elapsed_time: f32,
}

impl World {
    pub fn new(width: f32, height: f32) -> Self {
        let mut creatures = Vec::new();
        let mut rng = thread_rng();

        // Initial creature placement
        for _ in 0..50 {
            let x = rng.gen::<f32>() * width;
            let y = rng.gen::<f32>() * height;
            creatures.push(Creature::new(na::Point2::new(x, y)));
        }

        World {
            creatures,
            food_manager: FoodManager::new((width, height)),
            world_bounds: (width, height),
            generation: 0,
            elapsed_time: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.elapsed_time += dt;
        self.food_manager.update();

        let mut dead_creatures = Vec::new();
        let mut reproduction_events = Vec::new();
        let mut creature_updates = Vec::new();

        // First pass: Gather all information
        for i in 0..self.creatures.len() {
            let creature = &self.creatures[i];
            let nearby_foods = self
                .food_manager
                .find_nearby_food(&creature.physics.position, 200.0);

            let nearby_creatures: Vec<(usize, na::Point2<f32>, Gender, f32, f32)> = self
                .creatures
                .iter()
                .enumerate()
                .filter(|(j, other)| {
                    *j != i
                        && creature
                            .physics
                            .distance_to(&other.physics.position, self.world_bounds)
                            < 100.0
                })
                .map(|(j, other)| {
                    (
                        j,
                        other.physics.position,
                        other.gender,
                        other.physics.energy,
                        other.fitness,
                    )
                })
                .collect();

            creature_updates.push((i, nearby_foods, nearby_creatures));
        }

        // Second pass: Apply updates
        for (i, nearby_foods, nearby_creatures) in creature_updates {
            let creature = &mut self.creatures[i];
            let food_positions: Vec<na::Point2<f32>> =
                nearby_foods.iter().map(|(_, food)| food.position).collect();

            // Update creature
            creature.update(dt, &food_positions, &nearby_creatures, self.world_bounds);

            // Handle food consumption
            for (food_idx, _) in &nearby_foods {
                if creature.physics.energy < 1.0 {
                    creature.physics.energy += 0.2;
                    creature.physics.energy = creature.physics.energy.min(1.0);
                    self.food_manager.remove_food(*food_idx);
                }
            }

            // Check reproduction
            if let Some(mate_data) = nearby_creatures
                .iter()
                .find(|other| creature.can_reproduce_with(other))
            {
                let (mate_idx, _, _, _, _) = *mate_data;
                reproduction_events.push((i, mate_idx));
                creature.reproduction_cooldown = 15.0;
                creature.physics.energy -= 0.2;
            }

            // Check death condition
            if creature.physics.energy <= -0.2 {
                dead_creatures.push(i);
            }
        }

        // Process reproduction events
        let mut new_creatures = Vec::new();
        for (parent1_idx, parent2_idx) in reproduction_events {
            let parent1 = &self.creatures[parent1_idx];
            let parent2 = &self.creatures[parent2_idx];
            let mut child = parent1.reproduce_with(parent2);
            child.physics.position = parent1.physics.position;
            new_creatures.push(child);
        }

        // Add new creatures
        self.creatures.extend(new_creatures);

        // Remove dead creatures
        for &idx in dead_creatures.iter().rev() {
            self.creatures.remove(idx);
        }

        // Population management
        if self.creatures.len() < 10 {
            let mut rng = thread_rng();
            self.generation += 1;

            while self.creatures.len() < 50 {
                let x = rng.gen::<f32>() * self.world_bounds.0;
                let y = rng.gen::<f32>() * self.world_bounds.1;
                self.creatures.push(Creature::new(na::Point2::new(x, y)));
            }
        }

        self.food_manager.update_positions();
    }

    #[allow(dead_code)]
    pub fn resize(&mut self, width: f32, height: f32) {
        let old_bounds = self.world_bounds;
        self.world_bounds = (width, height);

        for creature in &mut self.creatures {
            creature.physics.position.x = (creature.physics.position.x / old_bounds.0) * width;
            creature.physics.position.y = (creature.physics.position.y / old_bounds.1) * height;
        }

        self.food_manager.resize(width, height);
    }
}
