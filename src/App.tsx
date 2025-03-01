import { useEffect, useRef, useState, useLayoutEffect } from 'react';
import { initializeSimulation } from './core/world/simulation';
import ControlsPanel from './components/ControlsPanel';
import StatsPanel from './components/StatsPanel';
import CreatureInfo from './components/CreatureInfo';

function App() {
  const canvasRef = useRef<HTMLDivElement>(null);
  const simulationRef = useRef<any>(null);
  const initializationAttempted = useRef<boolean>(false);
  const initializationPromise = useRef<Promise<any> | null>(null);
  const [isPaused, setIsPaused] = useState(false);
  const [isInitializing, setIsInitializing] = useState(true);
  const [initError, setInitError] = useState<string | null>(null);
  const [stats, setStats] = useState({
    fps: 0,
    creatureCount: 0,
    foodCount: 0,
    generation: 0,
    elapsedTime: 0,
  });
  const [selectedCreature, setSelectedCreature] = useState<any>(null);
  const [simulationParams, setSimulationParams] = useState({
    mutationRate: 0.05,
    foodSpawnRate: 0.5
  });

  // Ensure canvas container is mounted before initializing
  useLayoutEffect(() => {
    // Remove unused destructuring
    if (canvasRef.current) {
      canvasRef.current.getBoundingClientRect();
    }
  }, []);

  useEffect(() => {
    // Prevent double initialization in strict mode
    if (initializationAttempted.current) {
      return;
    }

    if (!canvasRef.current) {
      console.warn('Canvas container not mounted yet');
      return;
    }

    const initSimulation = async () => {
      try {
        // If initialization is already in progress, wait for it
        if (initializationPromise.current) {
          return await initializationPromise.current;
        }

        console.log('Starting simulation initialization');
        setIsInitializing(true);
        initializationAttempted.current = true;

        // Create initialization promise
        initializationPromise.current = (async () => {
          const simulation = await initializeSimulation(canvasRef.current!);
          console.log('Simulation initialized successfully');
          
          simulationRef.current = simulation;
          
          // Set the callback to update the selected creature
          simulation.setSelectedCreatureCallback((creature) => {
            setSelectedCreature(creature);
          });
          
          setIsInitializing(false);
          return simulation;
        })();

        await initializationPromise.current;
      } catch (error) {
        console.error('Failed to initialize simulation:', error);
        setInitError(error instanceof Error ? error.message : 'Failed to initialize simulation');
        setIsInitializing(false);
        initializationAttempted.current = false; // Allow retry on error
        initializationPromise.current = null;
      }
    };

    // Start initialization
    initSimulation().catch(error => {
      console.error('Unhandled error during initialization:', error);
    });
    
    // Stats update interval
    const statsInterval = setInterval(() => {
      if (simulationRef.current) {
        setStats(simulationRef.current.getStats());
      }
    }, 1000);
    
    return () => {
      clearInterval(statsInterval);
      if (simulationRef.current) {
        simulationRef.current.cleanup();
        simulationRef.current = null;
      }
      initializationAttempted.current = false;
      initializationPromise.current = null;
    };
  }, []);

  const handleTogglePause = () => {
    if (simulationRef.current) {
      const newPauseState = simulationRef.current.togglePause();
      setIsPaused(newPauseState);
    }
  };

  const handleReset = () => {
    // Reset simulation by reloading the page
    window.location.reload();
  };

  const handleMutationRateChange = (value: number) => {
    setSimulationParams(prev => ({
      ...prev,
      mutationRate: value
    }));
    // Update simulation parameters would go here when implemented
  };

  const handleFoodSpawnRateChange = (value: number) => {
    setSimulationParams(prev => ({
      ...prev,
      foodSpawnRate: value
    }));
    // Update simulation parameters would go here when implemented
  };

  return (
    <div style={{ width: '100%', height: '100%', position: 'relative' }}>
      <div ref={canvasRef} className="canvas-container" />
      {isInitializing ? (
        <div className="loading" style={{ 
          position: 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          padding: '20px',
          textAlign: 'center',
          color: '#666',
          backgroundColor: 'rgba(0, 0, 0, 0.7)',
          borderRadius: '8px',
          zIndex: 10
        }}>
          Initializing simulation...
        </div>
      ) : initError ? (
        <div className="error" style={{ 
          position: 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          padding: '20px',
          textAlign: 'center',
          color: 'red',
          backgroundColor: 'rgba(0, 0, 0, 0.7)',
          borderRadius: '8px',
          zIndex: 10
        }}>
          <div>Error: {initError}</div>
          <button 
            onClick={() => {
              setInitError(null);
              initializationAttempted.current = false;
            }}
            style={{
              marginTop: '10px',
              padding: '8px 16px',
              border: 'none',
              borderRadius: '4px',
              backgroundColor: '#4a90e2',
              color: 'white',
              cursor: 'pointer'
            }}
          >
            Retry
          </button>
        </div>
      ) : (
        <div className="ui-container" style={{}} data-testid="ui-container">
          <StatsPanel stats={stats} />
          <ControlsPanel 
            isPaused={isPaused} 
            onTogglePause={handleTogglePause}
            onReset={handleReset}
            mutationRate={simulationParams.mutationRate}
            foodSpawnRate={simulationParams.foodSpawnRate}
            onMutationRateChange={handleMutationRateChange}
            onFoodSpawnRateChange={handleFoodSpawnRateChange}
          />
          {selectedCreature ? (
            <CreatureInfo creature={selectedCreature} />
          ) : null}
        </div>
      )}
    </div>
  );
}

export default App;