import * as tf from '@tensorflow/tfjs';

// Initialize TensorFlow.js backend
let isModelInitialized = false;

export interface NeuralNetworkConfig {
  inputSize: number;
  outputSize: number;
  hiddenLayers?: number[];
  activationHidden?: string;
  activationOutput?: string;
}

/**
 * Neural network implementation using TensorFlow.js.
 * Handles creature brains with proper tensor management to prevent memory leaks.
 */
export class NeuralNetwork {
  private model: tf.Sequential;
  private config: NeuralNetworkConfig;
  private isDisposed = false;
  private isInitialized = false;

  constructor(config: NeuralNetworkConfig) {
    this.config = {
      inputSize: config.inputSize,
      outputSize: config.outputSize,
      hiddenLayers: config.hiddenLayers || [16, 16],
      activationHidden: config.activationHidden || 'relu',
      activationOutput: config.activationOutput || 'sigmoid'
    };
    
    // Create empty model (will be initialized in init())
    this.model = tf.sequential();
  }

  /**
   * Initialize the neural network. Must be called before using the network.
   */
  async init(): Promise<void> {
    if (this.isInitialized) return;

    try {
      // Ensure we're in a browser environment
      if (typeof window === 'undefined') {
        throw new Error('Neural network initialization requires a browser environment');
      }

      // Check if TensorFlow.js is properly loaded
      if (!tf || !tf.sequential) {
        throw new Error('TensorFlow.js is not properly loaded');
      }

      // Create model layers synchronously inside tidy
      tf.tidy(() => {
        console.log('Building neural network model');
        
        // Add first hidden layer
        this.model.add(tf.layers.dense({
          units: this.config.hiddenLayers![0],
          inputShape: [this.config.inputSize],
          activation: this.config.activationHidden,
          kernelInitializer: 'glorotNormal'
        }));

        // Add additional hidden layers if specified
        for (let i = 1; i < this.config.hiddenLayers!.length; i++) {
          this.model.add(tf.layers.dense({
            units: this.config.hiddenLayers![i],
            activation: this.config.activationHidden,
            kernelInitializer: 'glorotNormal'
          }));
        }

        // Add output layer
        this.model.add(tf.layers.dense({
          units: this.config.outputSize,
          activation: this.config.activationOutput,
          kernelInitializer: 'glorotNormal'
        }));
      });

      // Get model summary outside of tidy
      await this.model.summary();
      console.log('Neural network model constructed successfully');
      
      this.isInitialized = true;
    } catch (error) {
      console.error('Failed to initialize neural network:', error);
      this.isInitialized = false;
      throw error;
    }
  }

  /**
   * Predict output from input
   * @param inputs Array of input values
   * @returns Array of output values
   * @throws Error if the network has been disposed or not initialized
   */
  predict(inputs: number[]): number[] {
    if (this.isDisposed) {
      throw new Error('Cannot use a disposed neural network');
    }

    if (!this.isInitialized) {
      throw new Error('Neural network not initialized. Call init() first.');
    }

    return tf.tidy(() => {
      try {
        // Reshape inputs to match expected shape [1, inputSize]
        const inputTensor = tf.tensor2d([inputs], [1, this.config.inputSize]);
        
        // Get prediction
        const outputTensor = this.model.predict(inputTensor) as tf.Tensor;
        
        // Convert to array and return
        return Array.from(outputTensor.dataSync());
      } catch (error) {
        console.error('Error during neural network prediction:', error);
        // Return zeros as fallback
        return Array(this.config.outputSize).fill(0);
      }
    });
  }

  /**
   * Get a copy of the model weights as arrays
   * @throws Error if the network has been disposed
   */
  getWeights(): Float32Array[] {
    if (this.isDisposed) {
      throw new Error('Cannot get weights from a disposed neural network');
    }

    return tf.tidy(() => {
      const weights = this.model.getWeights();
      return weights.map(w => w.dataSync() as Float32Array);
    });
  }

  /**
   * Set weights to the model
   * @param weights Array of weight values
   * @throws Error if the network has been disposed
   */
  setWeights(weights: Float32Array[]): void {
    if (this.isDisposed) {
      throw new Error('Cannot set weights on a disposed neural network');
    }

    tf.tidy(() => {
      const originalWeights = this.model.getWeights();
      
      // Verify that weights array matches the expected length
      if (weights.length !== originalWeights.length) {
        throw new Error(`Weight array length mismatch: expected ${originalWeights.length}, got ${weights.length}`);
      }

      const tensors = weights.map((w, i) => {
        // Get the shape of the original tensor
        const originalShape = originalWeights[i].shape;
        // Create a new tensor with the provided weights and original shape
        return tf.tensor(w, originalShape);
      });
      
      try {
        this.model.setWeights(tensors);
      } finally {
        // Clean up the temporary tensors we created
        tensors.forEach(tensor => {
          if (tensor && !tensor.isDisposed) {
            tensor.dispose();
          }
        });
      }
    });
  }

  /**
   * Create a clone of this neural network
   * @returns A new neural network with the same architecture and weights
   * @throws Error if the network has been disposed
   */
  clone(): NeuralNetwork {
    if (this.isDisposed) {
      throw new Error('Cannot clone a disposed neural network');
    }

    return tf.tidy(() => {
      const clone = new NeuralNetwork(this.config);
      const weights = this.getWeights();
      clone.setWeights(weights);
      return clone;
    });
  }

  /**
   * Create a mutated version of this neural network
   * @param mutationRate The probability of mutation per weight
   * @param mutationAmount The maximum amount to mutate each weight
   * @returns A new mutated neural network
   * @throws Error if the network has been disposed
   */
  mutate(mutationRate: number = 0.1, mutationAmount: number = 0.2): NeuralNetwork {
    if (this.isDisposed) {
      throw new Error('Cannot mutate a disposed neural network');
    }

    return tf.tidy(() => {
      const mutated = new NeuralNetwork(this.config);
      const weights = this.getWeights();
      const mutatedWeights: Float32Array[] = [];

      for (const layerWeights of weights) {
        const newLayerWeights = new Float32Array(layerWeights.length);
        
        for (let j = 0; j < layerWeights.length; j++) {
          if (Math.random() < mutationRate) {
            // Apply random mutation within range [-mutationAmount, mutationAmount]
            newLayerWeights[j] = layerWeights[j] + (Math.random() * 2 - 1) * mutationAmount;
          } else {
            newLayerWeights[j] = layerWeights[j];
          }
        }
        
        mutatedWeights.push(newLayerWeights);
      }

      mutated.setWeights(mutatedWeights);
      return mutated;
    });
  }

  /**
   * Create a child network from two parent networks
   * @param other The other parent neural network
   * @param crossoverRate The probability of taking a weight from the other parent
   * @param mutationRate The probability of mutation per weight
   * @param mutationAmount The maximum amount to mutate each weight
   * @returns A new child neural network
   * @throws Error if either network has been disposed
   */
  crossover(
    other: NeuralNetwork,
    crossoverRate: number = 0.5,
    mutationRate: number = 0.1,
    mutationAmount: number = 0.2
  ): NeuralNetwork {
    if (this.isDisposed || other.isDisposed) {
      throw new Error('Cannot perform crossover with a disposed neural network');
    }

    return tf.tidy(() => {
      const child = new NeuralNetwork(this.config);
      const thisWeights = this.getWeights();
      const otherWeights = other.getWeights();
      const childWeights: Float32Array[] = [];

      for (let i = 0; i < thisWeights.length; i++) {
        const thisLayerWeights = thisWeights[i];
        const otherLayerWeights = otherWeights[i];
        const childLayerWeights = new Float32Array(thisLayerWeights.length);
        
        for (let j = 0; j < thisLayerWeights.length; j++) {
          // Crossover from other parent with probability
          const baseWeight = Math.random() < crossoverRate ? otherLayerWeights[j] : thisLayerWeights[j];
          
          // Apply mutation with probability
          if (Math.random() < mutationRate) {
            childLayerWeights[j] = baseWeight + (Math.random() * 2 - 1) * mutationAmount;
          } else {
            childLayerWeights[j] = baseWeight;
          }
        }
        
        childWeights.push(childLayerWeights);
      }

      child.setWeights(childWeights);
      return child;
    });
  }

  /**
   * Check if this network has been disposed
   */
  isDisposedNetwork(): boolean {
    return this.isDisposed;
  }

  /**
   * Dispose all tensors and free memory
   */
  dispose(): void {
    if (!this.isDisposed) {
      tf.tidy(() => {
        try {
          this.model.dispose();
        } catch (error) {
          console.error('Error disposing neural network:', error);
        }
      });
      this.isDisposed = true;
      this.isInitialized = false;
    }
  }
}