import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
import * as tf from '@tensorflow/tfjs';
import { createCreature, breedCreatures, Creature } from '../creature/creature';
import { createFood, consumeFood, removeFood, Food } from '../food/food';
import { setupWorld } from './world';
import { checkFoodCollisions, checkCreatureCollisions, updatePositions } from '../physics/physics';

// Track initialization state
let isBackendInitialized = false;

/**
 * Try to initialize TensorFlow.js with the best available backend
 */
async function initializeTensorFlow(): Promise<void> {
  if (isBackendInitialized) return;

  console.log('Starting TensorFlow.js initialization');
  
  try {
    // First try to initialize with WebGL
    try {
      console.log('Attempting to initialize WebGL backend');
      await tf.setBackend('webgl');
      await tf.ready();
      console.log('Successfully initialized WebGL backend');
      isBackendInitialized = true;
      return;
    } catch (webglError) {
      console.warn('WebGL backend initialization failed:', webglError);
    }

    // If WebGL fails, try CPU backend
    try {
      console.log('Falling back to CPU backend');
      await tf.setBackend('cpu');
      await tf.ready();
      console.log('Successfully initialized CPU backend');
      isBackendInitialized = true;
      return;
    } catch (cpuError) {
      console.error('CPU backend initialization failed:', cpuError);
    }

    throw new Error('Could not initialize any TensorFlow.js backend');
  } catch (error) {
    console.error('Failed to initialize TensorFlow.js:', error);
    throw error;
  }
}

/**
 * Initialize and run the simulation
 * @param container HTML element to render the simulation in
 * @returns Object with simulation control functions
 */
export async function initializeSimulation(container: HTMLDivElement) {
  try {
    console.log('Starting simulation initialization');
    
    // Validate container
    if (!container || !container.isConnected) {
      throw new Error('Invalid or disconnected container element');
    }
    console.log('Container validation passed');
    
    // Initialize TensorFlow.js first
    await initializeTensorFlow();
    
    // Create basic Three.js scene
    console.log('Creating Three.js scene');
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x161b33);
    
    // Add ambient light
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
    scene.add(ambientLight);
    
    // Add directional light
    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
    directionalLight.position.set(0, 10, 5);
    scene.add(directionalLight);

    // Get container dimensions
    const containerWidth = container.clientWidth || window.innerWidth;
    const containerHeight = container.clientHeight || window.innerHeight;
    
    console.log('Setting up camera and renderer');
    // Create camera
    const camera = new THREE.PerspectiveCamera(
      70, 
      containerWidth / containerHeight,
      0.1, 
      1000
    );
    camera.position.z = 15;

    // Create renderer with error handling
    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ 
        antialias: true,
        powerPreference: 'high-performance',
        alpha: true
      });
      
      // Test if WebGL context is working
      if (!renderer.getContext()) {
        throw new Error('WebGL context creation failed');
      }
      
      renderer.setSize(containerWidth, containerHeight);
      renderer.setPixelRatio(window.devicePixelRatio);
      
      // Clear any existing canvases
      while (container.firstChild) {
        container.removeChild(container.firstChild);
      }
      
      container.appendChild(renderer.domElement);
      console.log('Renderer initialized successfully');
    } catch (error) {
      console.error('Failed to create WebGL renderer:', error);
      throw new Error('WebGL initialization failed. Please check your browser settings.');
    }

    // Add orbit controls
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.05;
    controls.screenSpacePanning = true;
    controls.minDistance = 5;
    controls.maxDistance = 50;
    
    // Set camera for top-down 2D view
    controls.enableRotate = false;
    camera.position.set(0, 0, 30); // Increased Z distance for better overview
    camera.lookAt(0, 0, 0);
    camera.up.set(0, 1, 0); // Ensure correct up vector for top-down view
    
    // Initialize world settings
    const world = setupWorld(scene);
    
    // Initialize simulation state
    let isPaused = false;
    let lastTime = 0;
    let elapsedTime = 0;
    let frameCount = 0;
    let lastFpsUpdate = 0;
    let currentFps = 0;
    let generation = 1;
    
    // Initialize creatures and food
    const creatures: Creature[] = [];
    const foods: Food[] = [];
    
    // Initial population
    const INITIAL_CREATURE_COUNT = 20;
    const INITIAL_FOOD_COUNT = 50;
    const WORLD_SIZE = world.settings.size;
    
    // Keep track of active creatures to avoid using disposed ones
    const activeCreatures = new Set<string>();
    
    // Spawn initial creatures (now with Promise.all)
    const creaturePromises = [];
    for (let i = 0; i < INITIAL_CREATURE_COUNT; i++) {
      const x = (Math.random() - 0.5) * WORLD_SIZE;
      const y = (Math.random() - 0.5) * WORLD_SIZE;
      creaturePromises.push(createCreature(scene, { x, y }, 1));
    }
    
    // Wait for all creatures to be created and initialized
    const initialCreatures = await Promise.all(creaturePromises);
    creatures.push(...initialCreatures);
    
    // Spawn initial food
    for (let i = 0; i < INITIAL_FOOD_COUNT; i++) {
      const x = (Math.random() - 0.5) * WORLD_SIZE;
      const y = (Math.random() - 0.5) * WORLD_SIZE;
      const food = createFood(scene, { x, y }, world.settings.foodEnergy);
      foods.push(food);
    }
    
    // Selected creature tracking
    let selectedCreature: Creature | null = null;
    let selectedCreatureCallback: ((creature: Creature | null) => void) | null = null;
    
    // Handle window resize
    const handleResize = () => {
      const width = window.innerWidth;
      const height = window.innerHeight;
      
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
      renderer.setSize(width, height);
    };
    
    // Mouse interaction for selecting creatures
    const raycaster = new THREE.Raycaster();
    const mouse = new THREE.Vector2();
    
    const handleMouseDown = (event: MouseEvent) => {
      // Convert mouse position to normalized device coordinates
      mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
      mouse.y = -(event.clientY / window.innerHeight) * 2 + 1;
      
      raycaster.setFromCamera(mouse, camera);
      
      // Check for intersections with creatures
      const meshes = creatures.map(creature => creature.mesh);
      const intersects = raycaster.intersectObjects(meshes);
      
      // Handle right-click to deselect
      if (event.button === 2) {
        if (selectedCreature) {
          // Reset color of previously selected creature
          const material = selectedCreature.mesh.material as THREE.MeshStandardMaterial;
          material.color.setHex(selectedCreature.color);
        }
        
        if (selectedCreatureCallback) {
          selectedCreatureCallback(null);
        }
        selectedCreature = null;
        return;
      }
      
      // Left click to select
      if (intersects.length > 0) {
        const selectedMesh = intersects[0].object;
        const newSelectedCreature = creatures.find(creature => creature.mesh === selectedMesh) || null;
        
        // Reset color of previously selected creature
        if (selectedCreature) {
          const material = selectedCreature.mesh.material as THREE.MeshStandardMaterial;
          material.color.setHex(selectedCreature.color);
        }
        
        // Highlight newly selected creature
        if (newSelectedCreature) {
          const material = newSelectedCreature.mesh.material as THREE.MeshStandardMaterial;
          material.color.setHex(0xffff00); // Yellow highlight
        }
        
        if (newSelectedCreature && selectedCreatureCallback) {
          selectedCreatureCallback(newSelectedCreature);
        }
        selectedCreature = newSelectedCreature;
      }
    };
    
    // Keyboard controls
    const handleKeyDown = (event: KeyboardEvent) => {
      switch (event.key) {
        case ' ':
          // Space: Toggle pause
          togglePause();
          break;
        case 'r':
        case 'R':
          // R: Reset camera to top-down view
          camera.position.set(0, 0, 30);
          camera.lookAt(0, 0, 0);
          break;
      }
    };
    
    // Add event listeners
    window.addEventListener('resize', handleResize);
    renderer.domElement.addEventListener('mousedown', handleMouseDown);
    renderer.domElement.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', handleKeyDown);
    
    // Function to find the most fit creatures
    const findMostFitCreatures = (count: number): Creature[] => {
      const livingCreatures = creatures.filter(c => !c.isDead && activeCreatures.has(c.id));
      livingCreatures.sort((a, b) => b.fitness - a.fitness);
      return livingCreatures.slice(0, count);
    };
    
    // Function to dispose of dead creatures safely
    const disposeDeadCreatures = () => {
      const deadCreatures = creatures.filter(c => c.isDead);
      
      for (const creature of deadCreatures) {
        // Only dispose if it's still in our active set
        if (activeCreatures.has(creature.id)) {
          try {
            creature.dispose();
            activeCreatures.delete(creature.id);
          } catch (error) {
            console.error(`Error disposing creature ${creature.id}:`, error);
          }
        }
      }
    };
    
    // Function to spawn new generation of creatures
    const spawnNewGeneration = async () => {
      // Increment generation counter
      generation++;
      console.log(`Spawning generation ${generation}`);
      
      // Dispose dead creatures first
      disposeDeadCreatures();
      
      // Find the most fit creatures to use as parents
      const survivors = findMostFitCreatures(5);
      if (survivors.length < 2) {
        console.log('Not enough survivors for breeding, creating new random creatures');
        // Not enough survivors, create new random creatures
        const newCreaturePromises = [];
        for (let i = 0; i < INITIAL_CREATURE_COUNT; i++) {
          const x = (Math.random() - 0.5) * WORLD_SIZE;
          const y = (Math.random() - 0.5) * WORLD_SIZE;
          newCreaturePromises.push(createCreature(scene, { x, y }, generation));
        }
        const newCreatures = await Promise.all(newCreaturePromises);
        creatures.push(...newCreatures);
        return;
      }
      
      // Breed new generation
      const newGeneration: Creature[] = [];
      
      // Keep the survivors
      survivors.forEach(survivor => {
        // Reset survivor stats for new generation
        survivor.age = 0;
        survivor.energy = survivor.maxEnergy * 0.8;
        survivor.children = 0;
        newGeneration.push(survivor);
      });
      
      // Breed until we reach target population
      const breedingPromises = [];
      while (newGeneration.length + breedingPromises.length < INITIAL_CREATURE_COUNT) {
        // Pick two random parents from the survivors
        const parent1 = survivors[Math.floor(Math.random() * survivors.length)];
        const parent2 = survivors[Math.floor(Math.random() * survivors.length)];
        
        if (parent1 !== parent2) {
          try {
            // Random position for the child
            const x = (Math.random() - 0.5) * WORLD_SIZE;
            const y = (Math.random() - 0.5) * WORLD_SIZE;
            const childPromise = breedCreatures(scene, parent1, parent2, { x, y });
            breedingPromises.push(childPromise);
          } catch (error) {
            console.error('Error breeding creatures:', error);
            // If breeding fails, create a random creature instead
            const x = (Math.random() - 0.5) * WORLD_SIZE;
            const y = (Math.random() - 0.5) * WORLD_SIZE;
            const randomCreaturePromise = createCreature(scene, { x, y }, generation);
            breedingPromises.push(randomCreaturePromise);
          }
        }
      }
      
      // Wait for all breeding to complete
      const newCreatures = await Promise.all(breedingPromises);
      newGeneration.push(...newCreatures);
      
      // Remove old dead creatures that we haven't already disposed
      for (const creature of creatures) {
        if (creature.isDead && activeCreatures.has(creature.id)) {
          try {
            creature.dispose();
            activeCreatures.delete(creature.id);
          } catch (error) {
            console.error(`Error disposing creature ${creature.id}:`, error);
          }
        }
      }
      
      // Replace creatures array with new generation
      creatures.length = 0;
      creatures.push(...newGeneration);
      
      console.log(`New generation ${generation} spawned with ${creatures.length} creatures`);
    };
    
    // Animation loop
    const animate = (time: number) => {
      requestAnimationFrame(animate);
      
      // Calculate delta time
      const delta = Math.min((time - lastTime) / 1000, 0.1); // Cap delta to prevent large jumps
      lastTime = time;
      
      // Update FPS counter
      frameCount++;
      if (time - lastFpsUpdate > 1000) {
        currentFps = Math.round(frameCount / ((time - lastFpsUpdate) / 1000));
        frameCount = 0;
        lastFpsUpdate = time;
      }
      
      // Update controls
      controls.update();
      
      // Update simulation if not paused
      if (!isPaused) {
        elapsedTime += delta;
        
        // Update creature positions using physics engine
        updatePositions(
          creatures.filter(c => !c.isDead && activeCreatures.has(c.id)),
          delta,
          world.settings.size
        );
        
        // Update creatures' neural networks and behavior
        for (const creature of creatures) {
          // Skip dead or disposed creatures
          if (creature.isDead || !activeCreatures.has(creature.id)) continue;
          
          try {
            creature.update(delta, {
              creatures: creatures.filter(c => !c.isDead && activeCreatures.has(c.id)),
              foods: foods.filter(f => !f.isConsumed),
              settings: world.settings,
              getShortestDistance: world.getShortestDistance,
              wrapPosition: world.wrapPosition
            });
          } catch (error) {
            console.error(`Error updating creature ${creature.id}:`, error);
            // Mark creature as dead if update fails
            creature.isDead = true;
          }
        }
        
        // Check collisions between creatures
        checkCreatureCollisions(
          creatures.filter(c => !c.isDead && activeCreatures.has(c.id)),
          world.settings.size
        );
        
        // Check food collisions
        const consumedFoods = checkFoodCollisions(
          creatures.filter(c => !c.isDead && activeCreatures.has(c.id)),
          foods,
          world.settings.size,
          scene
        );
        
        // Remove consumed food
        const remainingFoods = foods.filter(food => !food.isConsumed);
        foods.length = 0;
        foods.push(...remainingFoods);
        
        // Spawn new food
        if (foods.length < world.settings.maxFoodCount && Math.random() < world.settings.foodSpawnRate * delta) {
          const x = (Math.random() - 0.5) * WORLD_SIZE;
          const y = (Math.random() - 0.5) * WORLD_SIZE;
          const food = createFood(scene, { x, y }, world.settings.foodEnergy);
          foods.push(food);
        }
        
        // Check which creatures want to reproduce
        const readyToReproduce: Creature[] = [];
        for (const creature of creatures) {
          if (
            !creature.isDead && 
            activeCreatures.has(creature.id) &&
            creature.energy > creature.maxEnergy * 0.6 &&
            Math.random() < 0.01 * delta
          ) {
            readyToReproduce.push(creature);
          }
        }
        
        // Handle reproduction
        for (const parent of readyToReproduce) {
          // Find another parent nearby
          let closestDistance = Infinity;
          let closestMate: Creature | null = null;
          
          for (const potentialMate of creatures) {
            if (
              potentialMate === parent || 
              potentialMate.isDead || 
              !activeCreatures.has(potentialMate.id)
            ) {
              continue;
            }
            
            const { distance } = world.getShortestDistance(parent.position, potentialMate.position);
            if (distance < closestDistance && distance < 3) {
              closestDistance = distance;
              closestMate = potentialMate;
            }
          }
          
          if (closestMate) {
            try {
              // Reduce energy of both parents
              parent.energy *= 0.7;
              closestMate.energy *= 0.7;
              parent.children++;
              closestMate.children++;
              
              // Create child nearby
              const childX = parent.position.x + (Math.random() * 2 - 1);
              const childY = parent.position.y + (Math.random() * 2 - 1);
              const child = breedCreatures(scene, parent, closestMate, { x: childX, y: childY });
              creatures.push(child);
              activeCreatures.add(child.id);
            } catch (error) {
              console.error('Error during reproduction:', error);
            }
          }
        }
        
        // Handle dead creatures
        for (const creature of creatures) {
          if (creature.isDead && activeCreatures.has(creature.id)) {
            // Fade out dead creatures
            const material = creature.mesh.material as THREE.MeshStandardMaterial;
            material.opacity = 0.3;
            material.transparent = true;
          }
        }
        
        // Periodically clean up disposed creatures
        if (Math.random() < 0.01) {
          disposeDeadCreatures();
        }
        
        // Reproduce/evolve if creature population is low
        const livingCreatures = creatures.filter(c => !c.isDead && activeCreatures.has(c.id));
        if (livingCreatures.length < INITIAL_CREATURE_COUNT / 3) {
          console.log('Population low, spawning new generation');
          spawnNewGeneration();
        }
        
        // If selected creature died or was disposed, deselect it
        if (
          selectedCreature && 
          (selectedCreature.isDead || !activeCreatures.has(selectedCreature.id))
        ) {
          if (selectedCreatureCallback) {
            selectedCreatureCallback(null);
          }
          selectedCreature = null;
        }
        
        // Focus camera on selected creature if exists
        if (
          selectedCreature && 
          !selectedCreature.isDead && 
          activeCreatures.has(selectedCreature.id)
        ) {
          camera.position.set(
            selectedCreature.position.x,
            selectedCreature.position.y,
            30 // Maintain top-down view height
          );
        }
      }
      
      // Render scene
      renderer.render(scene, camera);
    };
    
    // Start animation loop
    animate(0);
    
    // Cleanup function
    const cleanup = () => {
      console.log('Cleaning up simulation resources');
      
      window.removeEventListener('resize', handleResize);
      renderer.domElement.removeEventListener('mousedown', handleMouseDown);
      renderer.domElement.removeEventListener('contextmenu', (e) => e.preventDefault());
      window.removeEventListener('keydown', handleKeyDown);
      
      // Dispose of resources
      for (const creature of creatures) {
        if (activeCreatures.has(creature.id)) {
          try {
            creature.dispose();
            activeCreatures.delete(creature.id);
          } catch (error) {
            console.error(`Error disposing creature ${creature.id}:`, error);
          }
        }
      }
      
      for (const food of foods) {
        removeFood(food, scene);
      }
      
      // Dispose of Three.js resources
      renderer.dispose();
      
      // Clean up TensorFlow.js resources using the correct API
      try {
        // Safely dispose TensorFlow.js resources using available methods
        tf.disposeVariables();
        
        // Get memory info before cleanup attempt
        const memoryInfo = tf.memory();
        console.log('TensorFlow.js memory before cleanup:', memoryInfo);
        
        // Use the engine dispose method and garbage collection
        tf.engine().dispose();
        
        // Force garbage collection of any remaining tensors
        if (typeof tf.tidy === 'function') {
          tf.tidy(() => {
            // Empty tidy block to trigger cleanup
          });
        }
      } catch (error) {
        console.error('Error cleaning up TensorFlow resources:', error);
      }
    };
    
    // Toggle pause function
    const togglePause = () => {
      isPaused = !isPaused;
      return isPaused;
    };
    
    // Get stats function
    const getStats = (): SimulationStats => {
      return {
        fps: currentFps,
        creatureCount: creatures.filter(c => !c.isDead && activeCreatures.has(c.id)).length,
        foodCount: foods.filter(f => !f.isConsumed).length,
        generation,
        elapsedTime,
      };
    };
    
    // Set selected creature callback
    const setSelectedCreatureCallback = (callback: (creature: Creature | null) => void) => {
      selectedCreatureCallback = callback;
    };
    
    return {
      cleanup,
      togglePause,
      getStats,
      setSelectedCreatureCallback,
    };
  } catch (error) {
    console.error('Failed to initialize simulation:', error);
    throw error;
  }
}