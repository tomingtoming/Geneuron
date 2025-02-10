use ::rand::prelude::*;
use ::rand::thread_rng;
use std::f32::consts::PI;

pub trait Neural: Clone {
    fn process(&self, inputs: &[f32]) -> Vec<f32>;
    fn mutate(&mut self);
}

#[derive(Clone)]
pub struct FeedForwardNetwork {
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
    hidden_weights: Vec<f32>,
    output_weights: Vec<f32>,
}

impl FeedForwardNetwork {
    pub fn new(input_size: usize, output_size: usize) -> Self {
        let hidden_size = (input_size + output_size) * 2;
        let mut rng = thread_rng();

        let hidden_weights = (0..input_size * hidden_size)
            .map(|_| rng.gen::<f32>() * 2.0 - 1.0)
            .collect();

        let output_weights = (0..hidden_size * output_size)
            .map(|_| rng.gen::<f32>() * 2.0 - 1.0)
            .collect();

        FeedForwardNetwork {
            input_size,
            hidden_size,
            output_size,
            hidden_weights,
            output_weights,
        }
    }

    pub fn crossover_with(&self, other: &FeedForwardNetwork) -> FeedForwardNetwork {
        let mut child = FeedForwardNetwork::new(self.input_size, self.output_size);
        let mut rng = thread_rng();

        // Crossover hidden weights
        for i in 0..self.hidden_weights.len() {
            child.hidden_weights[i] = if rng.gen::<bool>() {
                self.hidden_weights[i]
            } else {
                other.hidden_weights[i]
            };
        }

        // Crossover output weights
        for i in 0..self.output_weights.len() {
            child.output_weights[i] = if rng.gen::<bool>() {
                self.output_weights[i]
            } else {
                other.output_weights[i]
            };
        }

        child
    }
}

impl Neural for FeedForwardNetwork {
    fn process(&self, inputs: &[f32]) -> Vec<f32> {
        assert_eq!(inputs.len(), self.input_size);

        // Process hidden layer
        let mut hidden = vec![0.0; self.hidden_size];
        for (i, h) in hidden.iter_mut().enumerate() {
            for (j, input) in inputs.iter().enumerate() {
                *h += input * self.hidden_weights[i * self.input_size + j];
            }
            *h = ((*h) * PI).tanh();
        }

        // Process output layer
        let mut outputs = vec![0.0; self.output_size];
        for (i, output) in outputs.iter_mut().enumerate() {
            for (j, h) in hidden.iter().enumerate() {
                *output += h * self.output_weights[i * self.hidden_size + j];
            }
            *output = ((*output) * PI).tanh();
        }

        outputs
    }

    fn mutate(&mut self) {
        let mut rng = thread_rng();
        let mutation_rate = 0.1;
        let mutation_range = 0.2;

        // Mutate hidden weights
        for weight in &mut self.hidden_weights {
            if rng.gen::<f32>() < mutation_rate {
                *weight += rng.gen::<f32>() * mutation_range * 2.0 - mutation_range;
                *weight = weight.clamp(-1.0, 1.0);
            }
        }

        // Mutate output weights
        for weight in &mut self.output_weights {
            if rng.gen::<f32>() < mutation_rate {
                *weight += rng.gen::<f32>() * mutation_range * 2.0 - mutation_range;
                *weight = weight.clamp(-1.0, 1.0);
            }
        }
    }
}
