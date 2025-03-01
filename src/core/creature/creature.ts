import * as THREE from 'three';
import { v4 as uuidv4 } from 'uuid';
import { NeuralNetwork } from '../neural/network';
import { Food, consumeFood } from '../food/food';

export interface CreatureConfig {
  position?: { x: number; y: number };
  generation?: number;
  energy?: number;
  neuralNetworkConfig?: {
    inputSize?: number;
    outputSize?: number;
    hiddenLayers?: number[];
  };
  color?: number;
  size?: number;
}

export interface Creature {
  id: string;
  mesh: THREE.Mesh;
  brain: NeuralNetwork;
  position: { x: number; y: number };
  velocity: { x: number; y: number };
  rotation: number;
  energy: number;
  maxEnergy: number;
  age: number;
  generation: number;
  fitness: number;
  children: number;
  isDead: boolean;
  color: number;
  size: number;
  update: (delta: number, world: any) => void;
  dispose: () => void;
}

/**
 * Creates a creature with a neural network brain that can interact with the environment
 * @param scene Three.js scene to add the creature to
 * @param position Initial position of the creature
 * @param generation Generation number of the creature
 * @param parentBrain Optional parent brain to inherit from (with mutation)
 * @returns A Promise that resolves to a new creature object
 */
export async function createCreature(
  scene: THREE.Scene,
  position = { x: 0, y: 0 },
  generation = 1,
  parentBrain?: NeuralNetwork
): Promise<Creature> {
  // Default configuration
  const config: CreatureConfig = {
    position,
    generation,
    energy: 50,
    neuralNetworkConfig: {
      inputSize: 8,  // Inputs: [closest food dx, closest food dy, energy, velocity x, velocity y, closest creature dx, closest creature dy, wall distance]
      outputSize: 3, // Outputs: [rotation change, acceleration, reproduce]
      hiddenLayers: [12, 12],
    },
    color: 0x3a7ca5,
    size: 0.5
  };
  
  // Create visual representation
  const geometry = new THREE.SphereGeometry(config.size!, 16, 12);
  const material = new THREE.MeshStandardMaterial({
    color: config.color!,
    emissive: 0x072940,
    emissiveIntensity: 0.2,
    roughness: 0.7,
  });
  const mesh = new THREE.Mesh(geometry, material);
  
  // Add visual indication of direction (a small "nose")
  const noseGeometry = new THREE.ConeGeometry(0.1, 0.3, 8);
  const noseMaterial = new THREE.MeshStandardMaterial({ color: 0xffffff });
  const nose = new THREE.Mesh(noseGeometry, noseMaterial);
  nose.rotation.x = Math.PI / 2;
  nose.position.set(0, 0, config.size! * 0.8);
  mesh.add(nose);
  
  // Add energy indicator ring
  const ringGeometry = new THREE.RingGeometry(config.size! * 1.2, config.size! * 1.3, 32);
  const ringMaterial = new THREE.MeshBasicMaterial({ 
    color: 0x00ff00,
    side: THREE.DoubleSide,
    transparent: true,
    opacity: 0.7
  });
  const ring = new THREE.Mesh(ringGeometry, ringMaterial);
  ring.rotation.x = -Math.PI / 2;
  mesh.add(ring);
  
  // Position the creature
  mesh.position.set(position.x, position.y, 0);
  scene.add(mesh);
  
  // Create neural network (brain)
  let brain: NeuralNetwork;
  
  if (parentBrain && !parentBrain.isDisposedNetwork()) {
    try {
      // Clone parent brain with mutation
      brain = parentBrain.mutate(0.1);
      await brain.init();
    } catch (error) {
      console.error('Error cloning parent brain, creating new one:', error);
      // Create new brain from scratch if there's an error
      brain = new NeuralNetwork({
        inputSize: config.neuralNetworkConfig!.inputSize!,
        outputSize: config.neuralNetworkConfig!.outputSize!,
        hiddenLayers: config.neuralNetworkConfig!.hiddenLayers,
      });
      await brain.init();
    }
  } else {
    // Create new brain from scratch
    brain = new NeuralNetwork({
      inputSize: config.neuralNetworkConfig!.inputSize!,
      outputSize: config.neuralNetworkConfig!.outputSize!,
      hiddenLayers: config.neuralNetworkConfig!.hiddenLayers,
    });
    await brain.init();
  }
  
  // Initial state
  const initialState = {
    id: uuidv4(),
    mesh,
    brain,
    position: { ...position },
    velocity: { x: 0, y: 0 },
    rotation: Math.random() * Math.PI * 2,
    energy: config.energy!,
    maxEnergy: config.energy! * 2,
    age: 0,
    generation,
    fitness: 0,
    children: 0,
    isDead: false,
    color: config.color!,
    size: config.size!,
  };
  
  // Create the creature object with update method
  const creature: Creature = {
    ...initialState,
    
    update(delta: number, world: any): void {
      // If already dead, don't update
      if (this.isDead) return;
      
      try {
        // Increase age
        this.age += delta;
        
        // Decrease energy over time (metabolism cost)
        this.energy -= delta * 2;
        
        // Die if no energy left
        if (this.energy <= 0) {
          this.isDead = true;
          return;
        }
        
        // Calculate fitness score (lifetime + energy gathered)
        this.fitness = this.age + (this.energy / 10);
        
        // Find closest food
        let closestFood: Food | null = null;
        let closestFoodDistance = Infinity;
        let closestFoodDx = 0;
        let closestFoodDy = 0;
        
        for (const food of world.foods) {
          if (food.isConsumed) continue;
          
          const { dx, dy, distance } = world.getShortestDistance(this.position, food.position);
          
          if (distance < closestFoodDistance) {
            closestFood = food;
            closestFoodDistance = distance;
            closestFoodDx = dx;
            closestFoodDy = dy;
          }
        }
        
        // Find closest creature
        let closestCreature: Creature | null = null;
        let closestCreatureDistance = Infinity;
        let closestCreatureDx = 0;
        let closestCreatureDy = 0;
        
        for (const otherCreature of world.creatures) {
          if (otherCreature === this || otherCreature.isDead) continue;
          
          const { dx, dy, distance } = world.getShortestDistance(this.position, otherCreature.position);
          
          if (distance < closestCreatureDistance) {
            closestCreature = otherCreature;
            closestCreatureDistance = distance;
            closestCreatureDx = dx;
            closestCreatureDy = dy;
          }
        }
        
        // Calculate distance to nearest wall
        const halfWorldSize = world.settings.size / 2;
        const distToWallX = Math.min(
          halfWorldSize - Math.abs(this.position.x),
          halfWorldSize + Math.abs(this.position.x)
        );
        const distToWallY = Math.min(
          halfWorldSize - Math.abs(this.position.y),
          halfWorldSize + Math.abs(this.position.y)
        );
        const wallDistance = Math.min(distToWallX, distToWallY);
        
        // Prepare inputs for neural network
        const inputs = [
          closestFoodDistance === Infinity ? 0 : closestFoodDx / world.settings.size,
          closestFoodDistance === Infinity ? 0 : closestFoodDy / world.settings.size,
          this.energy / this.maxEnergy,
          this.velocity.x / 5,
          this.velocity.y / 5,
          closestCreatureDistance === Infinity ? 0 : closestCreatureDx / world.settings.size,
          closestCreatureDistance === Infinity ? 0 : closestCreatureDy / world.settings.size,
          wallDistance / (world.settings.size / 2)
        ];
        
        // Get outputs from neural network
        let outputs;
        try {
          outputs = this.brain.predict(inputs);
        } catch (error) {
          console.error('Neural network prediction error:', error);
          // Default outputs if prediction fails
          outputs = [0.5, 0.5, 0];
        }
        
        const [rotationChange, acceleration, reproduction] = outputs;
        
        // Apply rotation change (map from 0-1 to -1 to 1)
        this.rotation += (rotationChange * 2 - 1) * delta * 3;
        
        // Apply acceleration
        const accelerationAmount = acceleration * delta * 10;
        this.velocity.x += Math.cos(this.rotation) * accelerationAmount;
        this.velocity.y += Math.sin(this.rotation) * accelerationAmount;
        
        // Apply friction
        const friction = 0.98;
        this.velocity.x *= friction;
        this.velocity.y *= friction;
        
        // Limit maximum velocity
        const maxVelocity = 5;
        const velocityMagnitude = Math.sqrt(
          this.velocity.x * this.velocity.x + this.velocity.y * this.velocity.y
        );
        
        if (velocityMagnitude > maxVelocity) {
          this.velocity.x = (this.velocity.x / velocityMagnitude) * maxVelocity;
          this.velocity.y = (this.velocity.y / velocityMagnitude) * maxVelocity;
        }
        
        // Move the creature
        this.position.x += this.velocity.x * delta;
        this.position.y += this.velocity.y * delta;
        
        // Handle world wrapping
        const { x, y } = world.wrapPosition(this.position);
        this.position.x = x;
        this.position.y = y;
        
        // Update mesh position and rotation
        this.mesh.position.set(this.position.x, this.position.y, 0);
        this.mesh.rotation.z = this.rotation;
        
        // Update energy ring color and scale
        const energyRatio = this.energy / this.maxEnergy;
        const ring = this.mesh.children[1] as THREE.Mesh;
        const ringMaterial = ring.material as THREE.MeshBasicMaterial;
        
        // Red to green based on energy level
        ringMaterial.color.setRGB(
          1 - energyRatio,  // Red component
          energyRatio,      // Green component
          0                 // Blue component
        );
        
        // Check for food collision and consumption
        if (closestFood && closestFoodDistance < this.size + 0.5) {
          // Consume food
          this.energy = Math.min(this.maxEnergy, this.energy + closestFood.energy);
          consumeFood(closestFood, scene);
        }
        
        // Handle reproduction
        if (reproduction > 0.8 && this.energy > this.maxEnergy * 0.6) {
          // Need significant energy and reproduction output signal to reproduce
          this.energy *= 0.6; // Reduce energy
          this.children++; // Increment child count
          
          // Creation of offspring handled by world controller
        }
      } catch (error) {
        console.error('Error in creature update:', error);
      }
    },
    
    dispose(): void {
      try {
        // Dispose neural network first
        if (brain && !brain.isDisposedNetwork()) {
          brain.dispose();
        }
      } catch (error) {
        console.error('Error disposing brain:', error);
      }
      
      try {
        // Remove from scene
        scene.remove(this.mesh);
        
        // Clean up geometry and materials
        if (this.mesh.geometry) this.mesh.geometry.dispose();
        
        // Clean up child geometries and materials
        this.mesh.children.forEach(child => {
          if (child instanceof THREE.Mesh) {
            if (child.geometry) child.geometry.dispose();
            if (child.material) {
              if (Array.isArray(child.material)) {
                child.material.forEach(material => material.dispose());
              } else {
                child.material.dispose();
              }
            }
          }
        });
        
        // Clean up main mesh material
        if (Array.isArray(this.mesh.material)) {
          this.mesh.material.forEach(material => material.dispose());
        } else if (this.mesh.material) {
          this.mesh.material.dispose();
        }
      } catch (error) {
        console.error('Error disposing creature mesh:', error);
      }
    }
  };
  
  return creature;
}

/**
 * Create a child creature by breeding two parents
 * @param scene Three.js scene to add the creature to
 * @param parent1 First parent creature
 * @param parent2 Second parent creature
 * @param position Optional position override
 * @returns A Promise that resolves to a new child creature
 */
export async function breedCreatures(
  scene: THREE.Scene,
  parent1: Creature,
  parent2: Creature,
  position?: { x: number; y: number }
): Promise<Creature> {
  // If no position provided, place near one of the parents
  const pos = position || {
    x: parent1.position.x + (Math.random() * 2 - 1),
    y: parent1.position.y + (Math.random() * 2 - 1)
  };
  
  // Safely create child with neural network based on crossover of parents
  let childBrain: NeuralNetwork;
  
  try {
    if (parent1.brain.isDisposedNetwork() || parent2.brain.isDisposedNetwork()) {
      throw new Error('Cannot breed with disposed brain');
    }
    
    childBrain = parent1.brain.crossover(parent2.brain);
    await childBrain.init();
  } catch (error) {
    console.error('Error during breeding, creating random brain:', error);
    // Create a fresh brain if crossover fails
    childBrain = new NeuralNetwork({
      inputSize: 8,
      outputSize: 3,
      hiddenLayers: [12, 12],
    });
    await childBrain.init();
  }
  
  // Determine color as mix of parents, with slight mutation
  const colorMix = (parent1.color + parent2.color) / 2;
  const colorMutation = Math.random() * 0.2 - 0.1; // -0.1 to 0.1
  const childColor = Math.max(0, Math.min(0xFFFFFF, colorMix * (1 + colorMutation)));
  
  // Create a child with generation+1
  const generation = Math.max(parent1.generation, parent2.generation) + 1;
  
  return await createCreature(
    scene,
    pos,
    generation,
    childBrain
  );
}