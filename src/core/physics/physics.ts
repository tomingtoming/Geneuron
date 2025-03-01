import * as THREE from 'three';
import { Creature } from '../creature/creature';
import { Food } from '../food/food';

/**
 * Check if two objects are colliding
 * @param obj1 First object with position and size/radius
 * @param obj2 Second object with position and size/radius
 * @param worldSize Size of the world for wrapping calculation
 */
export function checkCollision(
  obj1: { position: { x: number; y: number }, size?: number, radius?: number },
  obj2: { position: { x: number; y: number }, size?: number, radius?: number },
  worldSize: number
): boolean {
  const radius1 = obj1.size || obj1.radius || 0.5;
  const radius2 = obj2.size || obj2.radius || 0.5;
  
  // Calculate direct distance
  const dx = obj2.position.x - obj1.position.x;
  const dy = obj2.position.y - obj1.position.y;
  
  // Check for collisions considering world wrapping
  const directDistance = Math.sqrt(dx * dx + dy * dy);
  
  // Check if collision occurs in direct path
  if (directDistance < radius1 + radius2) {
    return true;
  }
  
  // Check for collisions across world edges
  const halfSize = worldSize / 2;
  
  // Calculate wrapped distances in each direction
  const wrapX = dx > 0 ? dx - worldSize : dx + worldSize;
  const wrapY = dy > 0 ? dy - worldSize : dy + worldSize;
  
  // Check X-wrapped distance
  const xWrappedDistance = Math.sqrt(wrapX * wrapX + dy * dy);
  if (xWrappedDistance < radius1 + radius2) {
    return true;
  }
  
  // Check Y-wrapped distance
  const yWrappedDistance = Math.sqrt(dx * dx + wrapY * wrapY);
  if (yWrappedDistance < radius1 + radius2) {
    return true;
  }
  
  // Check XY-wrapped distance
  const xyWrappedDistance = Math.sqrt(wrapX * wrapX + wrapY * wrapY);
  if (xyWrappedDistance < radius1 + radius2) {
    return true;
  }
  
  return false;
}

/**
 * Update positions of all creatures based on their velocities
 * @param creatures Array of creatures to update
 * @param delta Time delta since last update
 * @param worldSize Size of world for wrapping calculation
 */
export function updatePositions(creatures: Creature[], delta: number, worldSize: number): void {
  const halfSize = worldSize / 2;
  
  for (const creature of creatures) {
    if (creature.isDead) continue;
    
    // Update position based on velocity
    creature.position.x += creature.velocity.x * delta;
    creature.position.y += creature.velocity.y * delta;
    
    // Apply world wrapping
    if (creature.position.x > halfSize) {
      creature.position.x -= worldSize;
    } else if (creature.position.x < -halfSize) {
      creature.position.x += worldSize;
    }
    
    if (creature.position.y > halfSize) {
      creature.position.y -= worldSize;
    } else if (creature.position.y < -halfSize) {
      creature.position.y += worldSize;
    }
    
    // Update mesh position
    creature.mesh.position.set(creature.position.x, creature.position.y, 0);
    creature.mesh.rotation.z = creature.rotation;
  }
}

/**
 * Check for collisions between creatures and food
 * @param creatures Array of creatures
 * @param foods Array of food items
 * @param worldSize Size of the world
 * @param scene Three.js scene for visual updates
 * @returns Array of foods that were consumed
 */
export function checkFoodCollisions(
  creatures: Creature[],
  foods: Food[],
  worldSize: number,
  scene: THREE.Scene
): Food[] {
  const consumedFoods: Food[] = [];
  
  for (const creature of creatures) {
    if (creature.isDead) continue;
    
    for (const food of foods) {
      if (food.isConsumed) continue;
      
      if (checkCollision(creature, food, worldSize)) {
        // Food is consumed
        creature.energy = Math.min(creature.maxEnergy, creature.energy + food.energy);
        food.isConsumed = true;
        consumedFoods.push(food);
        
        // Scale down the food mesh (visual effect)
        const scale = 0.1;
        food.mesh.scale.set(scale, scale, scale);
        
        // Remove from scene
        scene.remove(food.mesh);
        
        // Dispose of geometry and materials
        food.mesh.geometry.dispose();
        if (Array.isArray(food.mesh.material)) {
          food.mesh.material.forEach(m => m.dispose());
        } else if (food.mesh.material) {
          food.mesh.material.dispose();
        }
      }
    }
  }
  
  return consumedFoods;
}

/**
 * Check for collisions between creatures
 * @param creatures Array of creatures
 * @param worldSize Size of the world
 */
export function checkCreatureCollisions(creatures: Creature[], worldSize: number): void {
  for (let i = 0; i < creatures.length; i++) {
    const creatureA = creatures[i];
    if (creatureA.isDead) continue;
    
    for (let j = i + 1; j < creatures.length; j++) {
      const creatureB = creatures[j];
      if (creatureB.isDead) continue;
      
      if (checkCollision(creatureA, creatureB, worldSize)) {
        // Simple elastic collision
        const tempVelocityX = creatureA.velocity.x;
        const tempVelocityY = creatureA.velocity.y;
        
        creatureA.velocity.x = creatureB.velocity.x * 0.8;
        creatureA.velocity.y = creatureB.velocity.y * 0.8;
        
        creatureB.velocity.x = tempVelocityX * 0.8;
        creatureB.velocity.y = tempVelocityY * 0.8;
        
        // Add a small random component to prevent creatures from getting stuck
        creatureA.velocity.x += (Math.random() - 0.5) * 0.2;
        creatureA.velocity.y += (Math.random() - 0.5) * 0.2;
        creatureB.velocity.x += (Math.random() - 0.5) * 0.2;
        creatureB.velocity.y += (Math.random() - 0.5) * 0.2;
      }
    }
  }
}