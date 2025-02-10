use nalgebra as na;
use rand::prelude::*;

use crate::creature::{Creature, Gender};
use crate::neural::FeedForwardNetwork;
use crate::food::FoodManager;

pub struct World {
    pub creatures: Vec<Creature>,
    pub generation: usize,
    pub elapsed_time: f32,
    pub food_manager: FoodManager,
    pub world_bounds: (f32, f32),
    repopulation_timer: f32,
    population_check_interval: f32,
}

#[allow(dead_code)]
impl World {
    pub fn new(width: f32, height: f32) -> Self {
        let world_bounds = (width, height);

        // 広い世界に合わせて初期生物数を増やす
        let creatures = (0..150).map(|_| {  // 50から150に増加
            let brain = Box::new(FeedForwardNetwork::new(9, 4));
            let mut creature = Creature::new(brain);
            creature.physics.position = na::Point2::new(
                rand::thread_rng().gen_range(0.0..width),
                rand::thread_rng().gen_range(0.0..height),
            );
            creature
        }).collect();

        // より多くの食物を生成
        let food_manager = FoodManager::new(world_bounds, 120, 150);  // 40,50から120,150に増加

        World {
            creatures,
            generation: 0,
            elapsed_time: 0.0,
            food_manager,
            world_bounds,
            repopulation_timer: 0.0,
            population_check_interval: 5.0,  // Check population every 5 seconds
        }
    }

    pub fn update(&mut self, dt: f32) {
        let mut dead_creatures = Vec::new();
        let mut reproduction_events = Vec::new();
        let mut food_to_remove = Vec::new();
        
        // Update reproduction cooldowns
        for creature in &mut self.creatures {
            if creature.reproduction_cooldown > 0.0 {
                creature.reproduction_cooldown -= dt;
            }
        }

        // Main update loop
        for i in 0..self.creatures.len() {
            // Create nearby creatures data
            let nearby_creatures: Vec<(usize, na::Point2<f32>, Gender, f32, f32)> = self.creatures.iter().enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(j, c)| (j, c.physics.position, c.gender.clone(), c.reproduction_cooldown, c.physics.energy))
                .collect();

            // Get mutable reference to current creature
            let creature = &mut self.creatures[i];
            
            // Update creature state
            let food_positions: Vec<na::Point2<f32>> = self.food_manager.foods.iter()
                .map(|food| food.position)
                .collect();
            creature.update(&food_positions, &nearby_creatures, dt, self.world_bounds);
            
            // Energy consumption with adjusted rates
            let energy_cost = creature.physics.calculate_energy_cost(dt);
            creature.physics.energy -= energy_cost;
            
            // Gradual energy regeneration when stationary
            if creature.physics.velocity.norm() < 1.0 {
                let rest_bonus = if nearby_creatures.iter().any(|(_, pos, ..)| 
                    na::distance(pos, &creature.physics.position) < 50.0) {
                    0.015 * dt  // Extra regeneration when resting near others
                } else {
                    0.01 * dt   // Normal regeneration when resting alone
                };
                creature.physics.energy += rest_bonus;
            }
            
            // Cap energy
            creature.physics.energy = creature.physics.energy.min(1.5);
            
            // トーラス構造の処理
            if creature.physics.position.x < 0.0 {
                creature.physics.position.x += self.world_bounds.0;
            } else if creature.physics.position.x > self.world_bounds.0 {
                creature.physics.position.x -= self.world_bounds.0;
            }
            if creature.physics.position.y < 0.0 {
                creature.physics.position.y += self.world_bounds.1;
            } else if creature.physics.position.y > self.world_bounds.1 {
                creature.physics.position.y -= self.world_bounds.1;
            }

            // Check death condition with grace period
            if creature.physics.energy <= -0.2 {
                dead_creatures.push(i);
                continue;
            }

            // Check reproduction with improved conditions
            if creature.reproduction_cooldown <= 0.0 && creature.physics.energy >= 0.7 {
                if let Some((mate_idx, _, _, _, _)) = nearby_creatures.iter()
                    .filter(|other| creature.can_reproduce_with(other, self.world_bounds))
                    .next()
                {
                    reproduction_events.push((i, *mate_idx));
                    creature.reproduction_cooldown = 15.0;
                    creature.physics.energy -= 0.2;
                }
            }
            
            // Check food consumption with improved positioning
            let nearby_foods = self.food_manager.find_nearby_food(&creature.physics.position, 20.0);
            for (food_idx, food) in nearby_foods {
                if !food_to_remove.contains(&food_idx) {  // 余分な括弧を削除
                    food_to_remove.push(food_idx);
                    creature.physics.energy += food.energy_value;
                    creature.fitness += 1.0;
                }
            }
        }

        // Handle reproduction
        let mut new_creatures = Vec::new();
        for (parent1_idx, parent2_idx) in reproduction_events {
            if parent1_idx < self.creatures.len() && parent2_idx < self.creatures.len() {
                let parent1 = self.creatures[parent1_idx].clone();
                let parent2 = self.creatures[parent2_idx].clone();
                let child = parent1.reproduce_with(&parent2);
                new_creatures.push(child);
            }
        }

        // Remove dead creatures
        dead_creatures.sort_unstable_by(|a, b| b.cmp(a));
        for &idx in &dead_creatures {
            if idx < self.creatures.len() {
                self.creatures.remove(idx);
            }
        }

        // Add new creatures
        self.creatures.extend(new_creatures);
        
        // Population management with timer
        self.repopulation_timer += dt;
        if self.repopulation_timer >= self.population_check_interval {
            self.repopulation_timer = 0.0;
            
            // Only add new creatures if population is critically low
            if self.creatures.len() < 10 {
                let current_pop = self.creatures.len();
                let max_new = (15 - current_pop).min(3);  // Add up to 3 at a time
                
                for _ in 0..max_new {
                    let brain = Box::new(FeedForwardNetwork::new(9, 4));
                    let mut new_creature = Creature::new(brain);
                    new_creature.physics.energy = 1.0;
                    
                    // Try to place new creatures near existing ones if possible
                    if let Some(existing) = self.creatures.choose(&mut thread_rng()) {
                        let mut rng = thread_rng();
                        new_creature.physics.position = na::Point2::new(
                            (existing.physics.position.x + rng.gen_range(-50.0..50.0))
                                .clamp(0.0, self.world_bounds.0),
                            (existing.physics.position.y + rng.gen_range(-50.0..50.0))
                                .clamp(0.0, self.world_bounds.1)
                        );
                    }
                    
                    self.creatures.push(new_creature);
                }
            }
        }
        
        // 最大生物数の制限を緩和
        if self.creatures.len() > 300 {  // 100から300に増加
            self.creatures.truncate(300);
        }
        
        // Handle food updates
        food_to_remove.sort_unstable_by(|a, b| b.cmp(a));
        food_to_remove.dedup();
        for &idx in food_to_remove.iter().rev() {
            self.food_manager.remove_food(idx);
        }
        
        // Update food system
        self.food_manager.update();
        
        // トーラス構造の処理（食物）
        for food in &mut self.food_manager.foods {
            if food.position.x < 0.0 {
                food.position.x += self.world_bounds.0;
            } else if food.position.x > self.world_bounds.0 {
                food.position.x -= self.world_bounds.0;
            }
            if food.position.y < 0.0 {
                food.position.y += self.world_bounds.1;
            } else if food.position.y > self.world_bounds.1 {
                food.position.y -= self.world_bounds.1;
            }
        }

        self.elapsed_time += dt;
        self.generation = (self.elapsed_time / 60.0) as usize + 1;  // New generation every minute
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        let old_bounds = self.world_bounds;
        self.world_bounds = (width, height);
        
        // 生物の位置を新しい境界に合わせてスケーリング
        for creature in &mut self.creatures {
            creature.physics.position.x = (creature.physics.position.x / old_bounds.0) * width;
            creature.physics.position.y = (creature.physics.position.y / old_bounds.1) * height;
        }
        
        // 食物マネージャーのリサイズを呼び出し
        self.food_manager.resize(width, height);
    }
}