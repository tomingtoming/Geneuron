use ::rand::Rng;
use ::rand::prelude::IteratorRandom;
use macroquad::prelude::*;
use nalgebra as na;

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
}

#[derive(Clone)]
pub struct FoodManager {
    pub foods: Vec<Food>,
    pub world_bounds: (f32, f32), // Make world_bounds public
    #[allow(dead_code)]
    min_food_count: usize,
    #[allow(dead_code)]
    max_food_count: usize,
    spawn_timer: f32,
    spawn_interval: f32,
}

impl FoodManager {
    pub fn new(world_bounds: (f32, f32)) -> Self {
        let max_food_count = 100;
        let min_food_count = 50;
        let mut foods = Vec::with_capacity(max_food_count);
        let mut rng = ::rand::rng();

        // Initial food placement
        for _ in 0..min_food_count {
            let x = rng.random_range(0.0..world_bounds.0);
            let y = rng.random_range(0.0..world_bounds.1);
            foods.push(Food::new(na::Point2::new(x, y)));
        }

        FoodManager {
            foods,
            max_food_count,
            min_food_count,
            world_bounds,
            spawn_timer: 0.0,
            spawn_interval: 0.1,
        }
    }

    #[allow(dead_code)]
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

    pub fn update(&mut self, dt: f32) {
        self.spawn_timer += dt;

        if self.spawn_timer >= self.spawn_interval {
            self.spawn_timer = 0.0;
            let mut rng = ::rand::rng();

            // 既存の食料の位置から新しい食料を生成（より自然な分布）
            if let Some(existing) = self.foods.iter().choose(&mut rng) {
                let x = (existing.position.x + rng.random_range(-50.0..50.0))
                    .rem_euclid(self.world_bounds.0);
                let y = (existing.position.y + rng.random_range(-50.0..50.0))
                    .rem_euclid(self.world_bounds.1);
                
                if self.foods.len() < 150 {  // 最大数を制限
                    self.foods.push(Food::new(na::Point2::new(x, y)));
                }
            } else if self.foods.is_empty() {
                // 食料が全くない場合はランダムな位置に生成
                let x = rng.random_range(0.0..self.world_bounds.0);
                let y = rng.random_range(0.0..self.world_bounds.1);
                self.foods.push(Food::new(na::Point2::new(x, y)));
            }
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

    #[allow(dead_code)]
    pub fn resize(&mut self, width: f32, height: f32) {
        // Scale food positions to new bounds
        for food in &mut self.foods {
            food.position.x = (food.position.x / self.world_bounds.0) * width;
            food.position.y = (food.position.y / self.world_bounds.1) * height;
        }
        self.world_bounds = (width, height);
    }

    #[allow(dead_code)]
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
