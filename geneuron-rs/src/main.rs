use ggez::{Context, GameResult};
use ggez::graphics::{self, Color, DrawParam, Mesh, Canvas, Text};
use ggez::event::{self, EventHandler};
use ggez::winit::event::VirtualKeyCode;
use nalgebra as na;
use rand::Rng;
use std::f32::consts::PI;

// 神経系のトレイト定義
trait NeuralNetwork {
    fn process(&self, inputs: &[f32]) -> Vec<f32>;
    fn mutate(&mut self, mutation_rate: f32);
}

// 単層ニューラルネットワーク
struct NeuronLayer {
    weights: na::DMatrix<f32>,
    bias: na::DVector<f32>,
}

impl NeuronLayer {
    fn new(inputs: usize, outputs: usize) -> Self {
        let mut rng = rand::thread_rng();
        NeuronLayer {
            weights: na::DMatrix::from_fn(inputs, outputs, |_, _| rng.gen_range(-1.0..1.0)),
            bias: na::DVector::from_fn(outputs, |_, _| rng.gen_range(-1.0..1.0)),
        }
    }

    fn process(&self, inputs: &[f32]) -> Vec<f32> {
        // Convert inputs to matrix
        let input_matrix = na::DMatrix::from_row_slice(1, inputs.len(), inputs);
        
        // Forward propagation
        let output = input_matrix * &self.weights + self.bias.transpose();
        
        // Apply activation function (sigmoid) and convert to Vec<f32>
        output.map(|x| 1.0 / (1.0 + (-x).exp())).row(0).iter().cloned().collect()
    }

    fn mutate(&mut self, mutation_rate: f32) {
        let mut rng = rand::thread_rng();
        
        // Mutate weights
        for weight in self.weights.iter_mut() {
            if rng.gen::<f32>() < mutation_rate {
                *weight += rng.gen_range(-0.5..0.5);
            }
        }
        
        // Mutate bias
        for bias in self.bias.iter_mut() {
            if rng.gen::<f32>() < mutation_rate {
                *bias += rng.gen_range(-0.5..0.5);
            }
        }
    }
}

impl NeuralNetwork for NeuronLayer {
    fn process(&self, inputs: &[f32]) -> Vec<f32> {
        self.process(inputs)
    }

    fn mutate(&mut self, mutation_rate: f32) {
        self.mutate(mutation_rate)
    }
}

// 生物の物理特性
#[derive(Clone)]
struct Physics {
    position: na::Point2<f32>,
    velocity: na::Vector2<f32>,
    rotation: f32,
    energy: f32,
}

// 生物の定義
struct Creature {
    physics: Physics,
    brain: Vec<NeuronLayer>,
    genome: Vec<f32>,
    color: Color,
    age: f32,
    fitness: f32,
}

impl Creature {
    fn think(&mut self, nearby_food: &[na::Point2<f32>]) {
        let mut inputs = Vec::with_capacity(5);
        
        // Current energy level
        inputs.push(self.physics.energy);
        
        // Current velocity
        let speed = (self.physics.velocity.x.powi(2) + self.physics.velocity.y.powi(2)).sqrt();
        inputs.push(speed / 100.0); // Normalized speed
        
        // Current rotation as normalized angle
        inputs.push(self.physics.rotation / (2.0 * PI));
        
        // Nearest food direction and distance
        if let Some(nearest) = self.find_nearest_food(nearby_food) {
            let direction = nearest - self.physics.position;
            let distance = direction.norm();
            inputs.push(distance / 800.0); // Normalize distance

            // Calculate angle between current rotation and food direction
            let target_angle = direction.y.atan2(direction.x);
            let angle_diff = (target_angle - self.physics.rotation + PI) % (2.0 * PI) - PI;
            inputs.push(angle_diff / PI); // Normalize angle difference
        } else {
            inputs.push(1.0); // Max distance if no food
            inputs.push(0.0); // Neutral angle if no food
        }
        
        // Process through neural network
        let outputs = self.brain[0].process(&inputs);
        
        // Apply decisions with smoother control
        let forward_speed = outputs[0] * 150.0; // Increased max speed
        let rotation_speed = (outputs[1] - 0.5) * 2.0 * PI; // Full rotation range
        
        // Smooth rotation
        self.physics.rotation += rotation_speed * 0.1; // Reduced rotation speed for smoother turning
        
        // Update velocity with inertia
        let target_velocity = na::Vector2::new(
            forward_speed * self.physics.rotation.cos(),
            forward_speed * self.physics.rotation.sin()
        );
        
        // Interpolate between current and target velocity (inertia)
        self.physics.velocity = self.physics.velocity * 0.9 + target_velocity * 0.1;
    }
    
    fn find_nearest_food(&self, food_sources: &[na::Point2<f32>]) -> Option<na::Point2<f32>> {
        food_sources.iter()
            .min_by(|a, b| {
                let dist_a = na::distance(&self.physics.position, a);
                let dist_b = na::distance(&self.physics.position, b);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .copied()
    }
}

// ワールドの状態管理
struct World {
    creatures: Vec<Creature>,
    generation: usize,
    elapsed_time: f32,
    food_sources: Vec<na::Point2<f32>>,
}

impl World {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        // Increased initial population
        let creatures = (0..50).map(|_| {
            Creature {
                physics: Physics {
                    position: na::Point2::new(rng.gen_range(0.0..800.0), rng.gen_range(0.0..600.0)),
                    velocity: na::Vector2::new(0.0, 0.0),
                    rotation: rng.gen_range(0.0..2.0 * PI),
                    energy: 1.0,
                },
                brain: vec![NeuronLayer::new(5, 4)],
                genome: vec![],
                color: Color::new(rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), 1.0),
                age: 0.0,
                fitness: 0.0,
            }
        }).collect();

        // Increased food sources
        let food_sources = (0..40).map(|_| {
            na::Point2::new(rng.gen_range(0.0..800.0), rng.gen_range(0.0..600.0))
        }).collect();

        World {
            creatures,
            generation: 0,
            elapsed_time: 0.0,
            food_sources,
        }
    }

    fn update(&mut self, dt: f32) {
        let mut food_to_remove = Vec::new();
        
        for creature in self.creatures.iter_mut() {
            // Neural network decision making
            creature.think(&self.food_sources);
            
            // Update physics with boundary check
            let new_pos = creature.physics.position + creature.physics.velocity * dt;
            
            // Boundary handling with smooth bounce
            if new_pos.x < 0.0 || new_pos.x > 800.0 {
                creature.physics.velocity.x *= -0.8; // Reduce speed on bounce
                creature.physics.position.x = new_pos.x.clamp(0.0, 800.0);
            } else {
                creature.physics.position.x = new_pos.x;
            }
            
            if new_pos.y < 0.0 || new_pos.y > 600.0 {
                creature.physics.velocity.y *= -0.8; // Reduce speed on bounce
                creature.physics.position.y = new_pos.y.clamp(0.0, 600.0);
            } else {
                creature.physics.position.y = new_pos.y;
            }
            
            // Energy consumption based on movement
            let speed = creature.physics.velocity.norm();
            let energy_cost = 0.1 * dt + speed * speed * 0.0001 * dt;
            creature.physics.energy -= energy_cost;
            
            // Check for food consumption
            const EATING_DISTANCE: f32 = 20.0; // Slightly increased eating range
            
            for (food_idx, food) in self.food_sources.iter().enumerate() {
                let distance = na::distance(&creature.physics.position, food);
                if distance < EATING_DISTANCE {
                    food_to_remove.push(food_idx);
                    creature.physics.energy += 0.5; // Increased energy gain
                    creature.fitness += 1.0;
                }
            }
        }
        
        // Remove eaten food and respawn
        food_to_remove.sort_unstable_by(|a, b| b.cmp(a));
        for food_idx in food_to_remove {
            self.food_sources.remove(food_idx);
        }
        
        // Respawn food if needed
        while self.food_sources.len() < 40 {
            let mut rng = rand::thread_rng();
            self.food_sources.push(na::Point2::new(
                rng.gen_range(0.0..800.0),
                rng.gen_range(0.0..600.0)
            ));
        }
        
        self.elapsed_time += dt;
    }
}

// メインのゲームステート
struct GameState {
    world: World,
    paused: bool,
    zoom: f32,
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // スペースキーでポーズ切り替え
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Space) {
            self.paused = !self.paused;
        }

        // ズーム制御
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Z) {
            self.zoom *= 1.05;
        }
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::X) {
            self.zoom *= 0.95;
        }

        if !self.paused {
            let dt = ctx.time.delta().as_secs_f32();
            self.update_world(dt)?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
        
        // ズーム適用
        canvas.set_screen_coordinates(graphics::Rect::new(
            0.0, 
            0.0, 
            800.0 / self.zoom, 
            600.0 / self.zoom,
        ));

        // 食料源の描画
        for food in &self.world.food_sources {
            let food_circle = Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [food.x, food.y],
                5.0,
                0.1,
                Color::GREEN,
            )?;
            canvas.draw(&food_circle, DrawParam::default());
        }

        // 生物の描画
        for creature in &self.world.creatures {
            // 生物の本体
            let body = Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [creature.physics.position.x, creature.physics.position.y],
                10.0,
                0.1,
                creature.color,
            )?;
            canvas.draw(&body, DrawParam::default());

            // 向きを示す線
            let direction_line = Mesh::new_line(
                ctx,
                &[
                    [creature.physics.position.x, creature.physics.position.y],
                    [
                        creature.physics.position.x + 20.0 * creature.physics.rotation.cos(),
                        creature.physics.position.y + 20.0 * creature.physics.rotation.sin()
                    ],
                ],
                2.0,
                Color::WHITE,
            )?;
            canvas.draw(&direction_line, DrawParam::default());
        }

        // 情報表示
        let info_text = Text::new(format!(
            "世代: {}\n生物数: {}\n経過時間: {:.1}秒\nFPS: {:.1}",
            self.world.generation,
            self.world.creatures.len(),
            self.world.elapsed_time,
            ctx.time.fps(),
        ));
        canvas.draw(
            &info_text,
            DrawParam::default()
                .color(Color::WHITE)
                .dest([10.0, 10.0]),
        );

        canvas.finish(ctx)?;
        Ok(())
    }
}

impl GameState {
    fn new(_ctx: &mut Context) -> GameResult<GameState> {
        Ok(GameState {
            world: World::new(),
            paused: false,
            zoom: 1.0,
        })
    }

    fn update_world(&mut self, dt: f32) -> GameResult {
        self.world.update(dt);
        Ok(())
    }
}

fn main() -> GameResult {
    // ゲーム設定
    let cb = ggez::ContextBuilder::new("geneuron", "neuroevolution")
        .window_setup(ggez::conf::WindowSetup::default().title("Geneuron-RS"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0));
    
    let (mut ctx, event_loop) = cb.build()?;
    
    // ゲームステートの作成と実行
    let state = GameState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
