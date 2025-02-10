use ggez::{Context, GameResult};
use ggez::graphics::{self, Color, DrawParam, Mesh, Canvas, Text};
use ggez::event::{self, EventHandler};
use ggez::winit::event::VirtualKeyCode;
use nalgebra as na;
use rand::Rng;
use std::f32::consts::PI;

// Window constants
const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

#[allow(dead_code)]  // Add attribute to suppress warning
// Neural network trait definition
trait NeuralNetwork {
    fn process(&self, inputs: &[f32]) -> Vec<f32>;
    fn mutate(&mut self, mutation_rate: f32);
    
    // Add default implementation for genome extraction
    fn extract_genome(&self) -> Vec<f32> {
        vec![]
    }
}

// Single layer neural network
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

    fn extract_genome(&self) -> Vec<f32> {
        let mut genome = Vec::new();
        for weight in self.weights.iter() {
            genome.push(*weight);
        }
        for bias in self.bias.iter() {
            genome.push(*bias);
        }
        genome
    }
}

impl NeuralNetwork for NeuronLayer {
    fn process(&self, inputs: &[f32]) -> Vec<f32> {
        self.process(inputs)
    }

    fn mutate(&mut self, mutation_rate: f32) {
        self.mutate(mutation_rate)
    }

    fn extract_genome(&self) -> Vec<f32> {
        self.extract_genome()
    }
}

// Add Clone implementation for NeuronLayer
impl Clone for NeuronLayer {
    fn clone(&self) -> Self {
        NeuronLayer {
            weights: self.weights.clone(),
            bias: self.bias.clone(),
        }
    }
}

// Creature physics properties
#[derive(Clone)]
struct Physics {
    position: na::Point2<f32>,
    velocity: na::Vector2<f32>,
    rotation: f32,
    energy: f32,
}

#[derive(Clone, PartialEq)]
enum Gender {
    Male,
    Female,
}

// Creature definition
#[derive(Clone)]
struct Creature {
    physics: Physics,
    brain: Vec<NeuronLayer>,
    genome: Vec<f32>,
    color: Color,
    age: f32,
    fitness: f32,
    gender: Gender,
    reproduction_cooldown: f32,  // Time until next reproduction is possible
    mode_color: Color,  // Added: Mode color indication
}

impl Creature {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let brain = vec![NeuronLayer::new(7, 4)];
        let genome = brain.iter()
            .flat_map(|layer| layer.extract_genome())
            .collect();

        Creature {
            physics: Physics {
                position: na::Point2::new(rng.gen_range(0.0..800.0), rng.gen_range(0.0..600.0)),
                velocity: na::Vector2::new(0.0, 0.0),
                rotation: rng.gen_range(0.0..2.0 * PI),
                energy: 1.0,
            },
            brain,
            genome,
            color: Color::new(rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), 1.0),
            age: 0.0,
            fitness: 0.0,
            gender: if rng.gen_bool(0.5) { Gender::Male } else { Gender::Female },
            reproduction_cooldown: 0.0,
            mode_color: Color::WHITE,  // Default mode color
        }
    }

    fn think(&mut self, nearby_food: &[na::Point2<f32>], nearby_creatures: &[(usize, na::Point2<f32>, Gender, f32, f32)]) {
        let mut inputs = Vec::with_capacity(7);
        
        // Current energy level and basic inputs
        inputs.push(self.physics.energy);
        inputs.push(self.physics.velocity.norm() / 150.0);  // Increased speed normalization
        inputs.push(self.physics.rotation / (2.0 * PI));
        
        // Find nearest mate with adjusted threshold
        let nearest_mate = nearby_creatures.iter()
            .filter(|other| self.can_reproduce_with(other))
            .map(|(_, pos, ..)| (pos, na::distance(&self.physics.position, pos)))
            .min_by(|(_, dist_a), (_, dist_b)| dist_a.partial_cmp(dist_b).unwrap());
        
        // Add mate-related inputs
        if let Some((mate_pos, distance)) = nearest_mate {
            let direction = mate_pos - self.physics.position;
            inputs.push(distance / 800.0);
            let target_angle = direction.y.atan2(direction.x);
            let angle_diff = (target_angle - self.physics.rotation + PI) % (2.0 * PI) - PI;
            inputs.push(angle_diff / PI);
        } else {
            inputs.push(1.0);
            inputs.push(0.0);
        }

        // Find nearest food with higher priority when hungry
        if let Some(nearest) = self.find_nearest_food(nearby_food) {
            let direction = nearest - self.physics.position;
            let distance = direction.norm();
            // Increase food priority when energy is low
            let adjusted_distance = if self.physics.energy < 0.3 {
                distance * 0.3  // Make food appear even closer when very hungry
            } else if self.physics.energy < 0.5 {
                distance * 0.7  // Still prioritize food when somewhat hungry
            } else {
                distance
            };
            inputs.push(adjusted_distance / 800.0);
            let target_angle = direction.y.atan2(direction.x);
            let angle_diff = (target_angle - self.physics.rotation + PI) % (2.0 * PI) - PI;
            inputs.push(angle_diff / PI);
        } else {
            inputs.push(1.0);
            inputs.push(0.0);
        }
        
        // Process through neural network and apply movement
        let outputs = self.brain[0].process(&inputs);
        
        // Enhanced speed control based on energy and situation
        let base_speed = outputs[0] * 200.0;  // Increased base speed
        let forward_speed = if self.physics.energy < 0.3 {
            base_speed * 0.5  // Conserve energy when very hungry
        } else if self.physics.energy < 0.5 {
            base_speed * 0.8  // Move slower when somewhat hungry
        } else if self.physics.energy > 1.2 {
            base_speed * 1.3  // Move faster when energy is abundant
        } else {
            base_speed
        };
        
        let rotation_speed = (outputs[1] - 0.5) * 3.0 * PI;  // Increased rotation speed
        
        // Smoother rotation with energy consideration
        let rotation_factor = if self.physics.energy < 0.3 { 0.2 } else { 0.15 };
        self.physics.rotation += rotation_speed * rotation_factor;
        
        // Update velocity with energy-based inertia
        let target_velocity = na::Vector2::new(
            forward_speed * self.physics.rotation.cos(),
            forward_speed * self.physics.rotation.sin()
        );
        
        // More responsive movement when energy is high
        let inertia_factor = if self.physics.energy > 1.0 {
            0.2  // Quicker response when energy is high
        } else if self.physics.energy < 0.3 {
            0.1  // Slower response when energy is low
        } else {
            0.15  // Normal response
        };
        self.physics.velocity = self.physics.velocity * (1.0 - inertia_factor) + target_velocity * inertia_factor;

        // Update mode color based on state
        self.mode_color = if self.physics.energy >= 0.7 && nearest_mate.is_some() {
            Color::RED     // Reproduction mode
        } else if self.physics.energy < 0.3 {
            Color::BLUE    // Hungry mode
        } else {
            Color::WHITE   // Normal mode
        };
    }

    fn can_reproduce_with(&self, other: &(usize, na::Point2<f32>, Gender, f32, f32)) -> bool {
        let (_, pos, gender, cooldown, energy) = other;
        *gender != self.gender &&
        *cooldown <= 0.0 &&
        *energy >= 0.7 &&
        self.reproduction_cooldown <= 0.0 &&
        self.physics.energy >= 0.7 &&
        na::distance(&self.physics.position, pos) < 30.0
    }

    fn reproduce_with(&self, other: &Creature) -> Creature {
        let mut child = Creature::new();
        let mut rng = rand::thread_rng();

        // Crossover using genomes
        let crossover_point = rng.gen_range(0..self.genome.len());
        child.genome = self.genome[..crossover_point].to_vec();
        child.genome.extend_from_slice(&other.genome[crossover_point..]);

        // Apply genome to brain weights and biases
        let mut genome_idx = 0;
        for layer in &mut child.brain {
            for weight in layer.weights.iter_mut() {
                if genome_idx < child.genome.len() {
                    *weight = child.genome[genome_idx];
                    genome_idx += 1;
                }
            }
            for bias in layer.bias.iter_mut() {
                if genome_idx < child.genome.len() {
                    *bias = child.genome[genome_idx];
                    genome_idx += 1;
                }
            }
        }

        // Inherit color
        child.color = Color::new(
            ((self.color.r + other.color.r) * 0.5 + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
            ((self.color.g + other.color.g) * 0.5 + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
            ((self.color.b + other.color.b) * 0.5 + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
            1.0
        );

        // Mutate
        child.mutate(0.1);

        // Set position near parents
        child.physics.position = na::Point2::new(
            (self.physics.position.x + other.physics.position.x) * 0.5 + rng.gen_range(-20.0..20.0),
            (self.physics.position.y + other.physics.position.y) * 0.5 + rng.gen_range(-20.0..20.0)
        );

        child
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

    fn mutate(&mut self, mutation_rate: f32) {
        let mut rng = rand::thread_rng();
        
        // Mutate genome
        for gene in &mut self.genome {
            if rng.gen::<f32>() < mutation_rate {
                *gene += rng.gen_range(-0.5..0.5);
            }
        }

        // Apply mutated genome to brain
        let mut genome_idx = 0;
        for layer in &mut self.brain {
            layer.mutate(mutation_rate);
            // Update genome with mutated values
            for weight in layer.weights.iter() {
                if genome_idx < self.genome.len() {
                    self.genome[genome_idx] = *weight;
                    genome_idx += 1;
                }
            }
            for bias in layer.bias.iter() {
                if genome_idx < self.genome.len() {
                    self.genome[genome_idx] = *bias;
                    genome_idx += 1;
                }
            }
        }
        
        // Mutate color
        if rng.gen::<f32>() < mutation_rate {
            self.color = Color::new(
                (self.color.r + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
                (self.color.g + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
                (self.color.b + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
                1.0,
            );
        }
    }
}

// World state management
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
        let creatures = (0..50).map(|_| Creature::new()).collect();

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
        let mut dead_creatures = Vec::new();
        let mut reproduction_events = Vec::new();
        
        // First pass: Update reproduction cooldowns and collect creature states
        for creature in &mut self.creatures {
            if creature.reproduction_cooldown > 0.0 {
                creature.reproduction_cooldown -= dt;
            }
        }

        // Second pass: Main update loop using indexed iteration
        let creature_count = self.creatures.len();
        for i in 0..creature_count {
            // Create nearby creatures data
            let nearby_creatures: Vec<(usize, na::Point2<f32>, Gender, f32, f32)> = self.creatures.iter().enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(j, c)| (j, c.physics.position, c.gender.clone(), c.reproduction_cooldown, c.physics.energy))
                .collect();

            // Get mutable reference to current creature
            let creature = &mut self.creatures[i];
            creature.age += dt;
            
            // Neural network decision making
            creature.think(&self.food_sources, &nearby_creatures);
            
            // Update physics and handle boundaries
            let new_pos = creature.physics.position + creature.physics.velocity * dt;
            
            // Boundary handling
            if new_pos.x < 0.0 || new_pos.x > 800.0 {
                creature.physics.velocity.x *= -0.8;
                creature.physics.position.x = new_pos.x.clamp(0.0, 800.0);
            } else {
                creature.physics.position.x = new_pos.x;
            }
            
            if new_pos.y < 0.0 || new_pos.y > 600.0 {
                creature.physics.velocity.y *= -0.8;
                creature.physics.position.y = new_pos.y.clamp(0.0, 600.0);
            } else {
                creature.physics.position.y = new_pos.y;
            }
            
            // Energy management
            let speed = creature.physics.velocity.norm();
            let energy_cost = 0.02 * dt + speed * speed * 0.00005 * dt;
            creature.physics.energy -= energy_cost;
            
            if speed < 1.0 {
                creature.physics.energy += 0.01 * dt;
            }
            
            creature.physics.energy = creature.physics.energy.min(1.5);
            
            // Check death condition
            if creature.physics.energy <= -0.2 {
                dead_creatures.push(i);
                continue;
            }

            // Handle reproduction
            if creature.reproduction_cooldown <= 0.0 && creature.physics.energy >= 0.7 {
                if let Some((mate_idx, _, _, _, _)) = nearby_creatures.iter()
                    .filter(|other| creature.can_reproduce_with(other))
                    .next()
                {
                    reproduction_events.push((i, *mate_idx));
                    creature.reproduction_cooldown = 15.0;
                    creature.physics.energy -= 0.2;
                }
            }
            
            // Food consumption
            let mut creature_food_indices = Vec::new();
            for (food_idx, food) in self.food_sources.iter().enumerate() {
                let distance = na::distance(&creature.physics.position, food);
                if distance < 20.0 {
                    creature_food_indices.push(food_idx);
                    creature.physics.energy += 0.8;
                    creature.fitness += 1.0;
                }
            }
            food_to_remove.extend(creature_food_indices);
        }

        // Handle reproduction events
        let mut new_creatures = Vec::new();
        for (parent1_idx, parent2_idx) in reproduction_events {
            // Double-check indices are still valid
            if parent1_idx < self.creatures.len() && parent2_idx < self.creatures.len() {
                // Clone parents before reproduction to avoid borrow checker issues
                let parent1 = self.creatures[parent1_idx].clone();
                let parent2 = self.creatures[parent2_idx].clone();
                let child = parent1.reproduce_with(&parent2);
                new_creatures.push(child);
            }
        }

        // Remove dead creatures (from highest index to lowest)
        dead_creatures.sort_unstable_by(|a, b| b.cmp(a));
        for &idx in &dead_creatures {
            // Only remove if index is still valid
            if idx < self.creatures.len() {
                self.creatures.remove(idx);
            }
        }

        // Add new creatures
        self.creatures.extend(new_creatures);
        
        // Maintain minimum population
        if self.creatures.len() < 10 {
            let needed = 10 - self.creatures.len();
            for _ in 0..needed {
                let mut new_creature = Creature::new();
                new_creature.physics.energy = 1.0;
                self.creatures.push(new_creature);
            }
        }
        
        // Handle food updates
        // Remove duplicates and sort in descending order
        food_to_remove.sort_unstable_by(|a, b| b.cmp(a));
        food_to_remove.dedup();
        
        // Remove food items from highest index to lowest
        for &idx in food_to_remove.iter() {
            if idx < self.food_sources.len() {
                self.food_sources.remove(idx);
            }
        }
        
        // Respawn food
        while self.food_sources.len() < 50 {
            let mut rng = rand::thread_rng();
            self.food_sources.push(na::Point2::new(
                rng.gen_range(0.0..800.0),
                rng.gen_range(0.0..600.0)
            ));
        }
        
        self.elapsed_time += dt;
    }
}

// Main game state
struct GameState {
    world: World,
    paused: bool,
    zoom: f32,
    window_size: (f32, f32),  // Added: Store window size
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Toggle pause with space key
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Space) {
            self.paused = !self.paused;
        }

        // Zoom control
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
        
        // Apply zoom based on window size
        canvas.set_screen_coordinates(graphics::Rect::new(
            0.0, 
            0.0, 
            self.window_size.0 / self.zoom, 
            self.window_size.1 / self.zoom,
        ));

        // Draw food sources
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

        // Draw creatures
        for creature in &self.world.creatures {
            // Creature body
            let body = Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [creature.physics.position.x, creature.physics.position.y],
                10.0,
                0.1,
                creature.color,
            )?;
            canvas.draw(&body, DrawParam::default());

            // Direction indicator with mode color
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
                creature.mode_color,  // Use mode color for direction line
            )?;
            canvas.draw(&direction_line, DrawParam::default());
        }

        // Display information
        let info_text = Text::new(format!(
            "Generation: {}\nCreatures: {}\nElapsed Time: {:.1}s\nFPS: {:.1}",
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

    // Added: Window resize event handler
    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.window_size = (width, height);
        Ok(())
    }
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        let (width, height) = ctx.gfx.drawable_size();
        Ok(GameState {
            world: World::new(),
            paused: false,
            zoom: 1.0,
            window_size: (width, height),
        })
    }

    fn update_world(&mut self, dt: f32) -> GameResult {
        self.world.update(dt);
        Ok(())
    }
}

fn main() -> GameResult {
    // Game configuration
    let cb = ggez::ContextBuilder::new("geneuron", "neuroevolution")
        .window_setup(ggez::conf::WindowSetup::default().title("Geneuron-RS"))
        .window_mode(ggez::conf::WindowMode::default()
            .dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)
            .resizable(true));  // Make window resizable
    
    let (mut ctx, event_loop) = cb.build()?;
    
    // Create and run game state
    let state = GameState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
