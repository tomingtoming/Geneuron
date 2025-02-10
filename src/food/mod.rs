use nalgebra as na;
use rand::prelude::*;
use ggez::graphics::Color;

#[derive(Clone)]
pub struct Food {
    pub position: na::Point2<f32>,
    pub energy_value: f32,
    pub size: f32,
    pub color: Color,
}

pub struct FoodManager {
    pub foods: Vec<Food>,
    pub world_bounds: (f32, f32),  // Make world_bounds public
    min_food_count: usize,
    max_food_count: usize,
}

impl Food {
    pub fn new(position: na::Point2<f32>) -> Self {
        Food {
            position,
            energy_value: 0.8,  // Default energy value
            size: 5.0,         // Default size
            color: Color::GREEN, // Default color
        }
    }

    pub fn distance_to(&self, point: &na::Point2<f32>) -> f32 {
        na::distance(&self.position, point)
    }
}

impl FoodManager {
    pub fn new(world_bounds: (f32, f32), min_count: usize, max_count: usize) -> Self {
        let mut manager = FoodManager {
            foods: Vec::new(),
            world_bounds,
            min_food_count: min_count,
            max_food_count: max_count,
        };
        manager.spawn_initial_food();
        manager
    }

    fn spawn_initial_food(&mut self) {
        let mut rng = thread_rng();
        for _ in 0..self.min_food_count {
            self.spawn_food_at(na::Point2::new(
                rng.gen_range(0.0..self.world_bounds.0),
                rng.gen_range(0.0..self.world_bounds.1)
            ));
        }
    }

    pub fn spawn_food_at(&mut self, position: na::Point2<f32>) {
        if self.foods.len() < self.max_food_count {
            self.foods.push(Food::new(position));
        }
    }

    pub fn remove_food(&mut self, index: usize) {
        if index < self.foods.len() {
            self.foods.remove(index);
        }
    }

    pub fn update(&mut self) {
        let mut rng = thread_rng();
        
        // Maintain minimum food count
        while self.foods.len() < self.min_food_count {
            self.spawn_food_at(na::Point2::new(
                rng.gen_range(0.0..self.world_bounds.0),
                rng.gen_range(0.0..self.world_bounds.1)
            ));
        }
        
        // Occasionally spawn extra food up to max_food_count
        if self.foods.len() < self.max_food_count && rng.gen::<f32>() < 0.1 {
            self.spawn_food_at(na::Point2::new(
                rng.gen_range(0.0..self.world_bounds.0),
                rng.gen_range(0.0..self.world_bounds.1)
            ));
        }
    }

    pub fn find_nearby_food(&self, position: &na::Point2<f32>, radius: f32) -> Vec<(usize, &Food)> {
        self.foods.iter().enumerate()
            .filter(|(_, food)| food.distance_to(position) <= radius)
            .collect()
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        // Scale food positions to new bounds
        for food in &mut self.foods {
            food.position.x = (food.position.x / self.world_bounds.0) * width;
            food.position.y = (food.position.y / self.world_bounds.1) * height;
        }
        self.world_bounds = (width, height);
    }
}