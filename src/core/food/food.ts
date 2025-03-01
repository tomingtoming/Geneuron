import * as THREE from 'three';

export interface Food {
  id: number;
  mesh: THREE.Mesh;
  position: { x: number; y: number };
  energy: number;
  isConsumed: boolean;
}

let nextId = 0;

export function createFood(
  scene: THREE.Scene, 
  position: { x: number; y: number },
  energy: number
): Food {
  const geometry = new THREE.SphereGeometry(0.3, 8, 6);
  const material = new THREE.MeshStandardMaterial({
    color: 0x00ff00,
    emissive: 0x002200,
    emissiveIntensity: 0.2,
    roughness: 0.7,
  });
  
  const mesh = new THREE.Mesh(geometry, material);
  mesh.position.set(position.x, position.y, 0);
  scene.add(mesh);
  
  return {
    id: nextId++,
    mesh,
    position,
    energy,
    isConsumed: false,
  };
}

export function removeFood(food: Food, scene: THREE.Scene): void {
  if (!food.isConsumed) {
    food.isConsumed = true;
    scene.remove(food.mesh);
    food.mesh.geometry.dispose();
    if (Array.isArray(food.mesh.material)) {
      food.mesh.material.forEach(material => material.dispose());
    } else {
      food.mesh.material.dispose();
    }
  }
}

export function consumeFood(food: Food, scene: THREE.Scene): void {
  removeFood(food, scene);
}