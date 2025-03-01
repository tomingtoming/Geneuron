import * as THREE from 'three';

export interface WorldSettings {
  size: number;
  gridSize: number;
  foodEnergy: number;
  maxFoodCount: number;
  foodSpawnRate: number;
  mutationRate: number;
  energyDecayRate: number;
  minEnergyToReproduce: number;
}

export function setupWorld(scene: THREE.Scene) {
  // Default world settings
  const settings: WorldSettings = {
    size: 50,
    gridSize: 100,
    foodEnergy: 10,
    maxFoodCount: 100,
    foodSpawnRate: 0.5,
    mutationRate: 0.05,
    energyDecayRate: 0.1,
    minEnergyToReproduce: 50
  };

  // Add a ground plane grid for reference
  const gridHelper = new THREE.GridHelper(settings.size, settings.gridSize, 0x444444, 0x222222);
  gridHelper.rotation.x = Math.PI / 2; // Rotate grid to XY plane for top-down view
  scene.add(gridHelper);

  // Add world boundaries visualization
  const boundaryGeometry = new THREE.BoxGeometry(settings.size, settings.size, 1);
  const boundaryEdges = new THREE.EdgesGeometry(boundaryGeometry);
  const boundaryLines = new THREE.LineSegments(
    boundaryEdges,
    new THREE.LineBasicMaterial({ color: 0x3a7ca5 })
  );
  boundaryLines.rotation.x = Math.PI / 2; // Align with grid
  scene.add(boundaryLines);

  // Methods to update world settings
  const updateSettings = (newSettings: Partial<WorldSettings>) => {
    Object.assign(settings, newSettings);
  };
  
  // Function to check if a position is within world boundaries
  const isWithinBounds = (x: number, y: number): boolean => {
    const halfSize = settings.size / 2;
    return x >= -halfSize && x <= halfSize && y >= -halfSize && y <= halfSize;
  };
  
  // Function to wrap position around toroidal world
  const wrapPosition = (position: { x: number; y: number }) => {
    const halfSize = settings.size / 2;
    
    // Wrap x coordinate
    if (position.x > halfSize) {
      position.x = -halfSize + (position.x - halfSize);
    } else if (position.x < -halfSize) {
      position.x = halfSize - (-halfSize - position.x);
    }
    
    // Wrap y coordinate
    if (position.y > halfSize) {
      position.y = -halfSize + (position.y - halfSize);
    } else if (position.y < -halfSize) {
      position.y = halfSize - (-halfSize - position.y);
    }
    
    return position;
  };
  
  // Calculate shortest distance considering world wrapping
  const getShortestDistance = (pos1: { x: number; y: number }, pos2: { x: number; y: number }) => {
    const halfSize = settings.size / 2;
    let dx = pos2.x - pos1.x;
    let dy = pos2.y - pos1.y;
    
    // Consider x-wrapping
    if (Math.abs(dx) > halfSize) {
      dx = dx > 0 ? dx - settings.size : dx + settings.size;
    }
    
    // Consider y-wrapping
    if (Math.abs(dy) > halfSize) {
      dy = dy > 0 ? dy - settings.size : dy + settings.size;
    }
    
    return { dx, dy, distance: Math.sqrt(dx * dx + dy * dy) };
  };
  
  return {
    settings,
    updateSettings,
    isWithinBounds,
    wrapPosition,
    getShortestDistance,
  };
}