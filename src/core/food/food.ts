import * as THREE from 'three';
import { v4 as uuidv4 } from 'uuid';

export interface Food {
  id: string;
  mesh: THREE.Mesh;
  position: { x: number; y: number };
  energy: number;
  isConsumed: boolean;
}

export function createFood(
  scene: THREE.Scene,
  position: { x: number; y: number },
  energy: number = 10
): Food {
  // Create visual representation for food
  const geometry = new THREE.SphereGeometry(0.3, 8, 8);
  const material = new THREE.MeshStandardMaterial({
    color: 0x88cc88,
    emissive: 0x226622,
    emissiveIntensity: 0.3,
  });
  const mesh = new THREE.Mesh(geometry, material);
  
  // Position the food
  mesh.position.set(position.x, position.y, 0);
  scene.add(mesh);
  
  // Create and return the food object
  return {
    id: uuidv4(),
    mesh,
    position,
    energy,
    isConsumed: false
  };
}

// Function to handle food consumption
export function consumeFood(food: Food, scene?: THREE.Scene): void {
  if (food.isConsumed) return;
  
  food.isConsumed = true;
  food.energy = 0;
  
  // Visual effect for consumption
  const scale = { value: 1.0 };
  const targetScale = 0.01;
  const duration = 300; // milliseconds
  const startTime = Date.now();
  
  function animateConsumption() {
    const elapsed = Date.now() - startTime;
    const progress = Math.min(elapsed / duration, 1);
    
    const newScale = 1.0 - progress * (1.0 - targetScale);
    food.mesh.scale.set(newScale, newScale, newScale);
    
    if (progress < 1) {
      requestAnimationFrame(animateConsumption);
    } else {
      // Remove from scene when animation completes
      if (scene) {
        scene.remove(food.mesh);
      }
      
      // Clean up geometry and material
      if (food.mesh.geometry) food.mesh.geometry.dispose();
      if (Array.isArray(food.mesh.material)) {
        food.mesh.material.forEach(material => material.dispose());
      } else if (food.mesh.material) {
        food.mesh.material.dispose();
      }
    }
  }
  
  animateConsumption();
}

// Function to remove food from the scene
export function removeFood(food: Food, scene: THREE.Scene): void {
  scene.remove(food.mesh);
  
  // Clean up resources
  if (food.mesh.geometry) food.mesh.geometry.dispose();
  if (Array.isArray(food.mesh.material)) {
    food.mesh.material.forEach(material => material.dispose());
  } else if (food.mesh.material) {
    food.mesh.material.dispose();
  }
}