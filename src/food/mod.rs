use macroquad::prelude::*;
use nalgebra as na;
use ::rand::prelude::*;
use ::rand::thread_rng;

#[derive(Clone)]
pub struct Food {
    pub position: na::Point2<f32>,
    pub size: f32,
    pub color: Color,
}

impl Food {
    pub fn new(position: na::Point2<f32>) -> Self {
        Food {
            position,
            size: 5.0,
            color: GREEN,
        }
    }

    pub fn distance_to(&self, point: &na::Point2<f32>) -> f32 {
        na::distance(&self.position, point)
    }
}

#[derive(Clone)]
pub struct FoodManager {
    pub foods: Vec<Food>,
    pub world_bounds: (f32, f32), // Make world_bounds public
    min_food_count: usize,
    max_food_count: usize,
}

impl FoodManager {
    pub fn new(world_bounds: (f32, f32)) -> Self {
        let max_food_count = 100;
        let min_food_count = 50;
        let mut foods = Vec::with_capacity(max_food_count);
        let mut rng = thread_rng();

        // Initial food placement
        for _ in 0..min_food_count {
            let x = rng.gen::<f32>() * world_bounds.0;
            let y = rng.gen::<f32>() * world_bounds.1;
            foods.push(Food::new(na::Point2::new(x, y)));
        }

        FoodManager {
            foods,
            max_food_count,
            min_food_count,
            world_bounds,
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
                rng.gen_range(0.0..self.world_bounds.1),
            ));
        }

        // Occasionally spawn extra food up to max_food_count
        if self.foods.len() < self.max_food_count && rng.gen::<f32>() < 0.1 {
            self.spawn_food_at(na::Point2::new(
                rng.gen_range(0.0..self.world_bounds.0),
                rng.gen_range(0.0..self.world_bounds.1),
            ));
        }
    }

    pub fn find_nearby_food(&self, position: &na::Point2<f32>, radius: f32) -> Vec<(usize, Food)> {
        self.foods
            .iter()
            .enumerate()
            .filter(|(_, food)| {
                let dx = (food.position.x - position.x).abs();
                let dy = (food.position.y - position.y).abs();
                let wrapped_dx = dx.min(self.world_bounds.0 - dx);
                let wrapped_dy = dy.min(self.world_bounds.1 - dy);
                (wrapped_dx * wrapped_dx + wrapped_dy * wrapped_dy).sqrt() < radius
            })
            .map(|(i, food)| (i, food.clone()))
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

    pub fn update_positions(&mut self) {
        for food in &mut self.foods {
            // トーラス構造の処理
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
    }
}
