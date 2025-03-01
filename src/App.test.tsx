import { render, screen, fireEvent, within, waitFor } from '@testing-library/react';
import { vi, describe, test, expect, beforeEach } from 'vitest';
import { act } from 'react';
import App from './App';
import { initializeSimulation } from './core/world/simulation';

// Mock the simulation module
vi.mock('./core/world/simulation', () => ({
  initializeSimulation: vi.fn().mockImplementation((container) => {
    return new Promise((resolve) => {
      let selectedCreatureCallback: ((creature: any) => void) | null = null;
      
      const simulation = {
        cleanup: vi.fn(),
        togglePause: vi.fn().mockReturnValue(true),
        getStats: vi.fn().mockReturnValue({
          fps: 60,
          creatureCount: 10,
          foodCount: 20,
          generation: 1,
          elapsedTime: 1000
        }),
        selectCreature: vi.fn().mockImplementation((creature) => {
          return new Promise<void>((resolveSelect) => {
            if (selectedCreatureCallback) {
              selectedCreatureCallback(creature);
            }
            resolveSelect();
          });
        }),
        setSelectedCreatureCallback: vi.fn().mockImplementation((callback) => {
          selectedCreatureCallback = callback;
          return () => {
            selectedCreatureCallback = null;
          };
        }),
      };
      setTimeout(() => resolve(simulation), 0);
    });
  })
}));

// Reset mocks between tests to avoid state leakage
beforeEach(() => {
  vi.clearAllMocks();
});

describe('App Component', () => {
  test('renders loading state initially', async () => {
    render(<App />);
    expect(screen.getByText('Initializing simulation...')).toBeInTheDocument();
  });

  test('renders UI components after initialization', async () => {
    render(<App />);
    
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
    });

    expect(screen.queryByText('Initializing simulation...')).not.toBeInTheDocument();
    expect(screen.getByText(/FPS:/i)).toBeInTheDocument();
    expect(screen.getByText(/Creatures:/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /pause/i })).toBeInTheDocument();
  });

  test('handles pause/resume functionality', async () => {
    render(<App />);
    
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
    });

    const pauseButton = screen.getByRole('button', { name: /pause/i });
    await act(async () => {
      fireEvent.click(pauseButton);
    });
    
    expect(screen.getByRole('button', { name: /resume/i })).toBeInTheDocument();
  });

  test('displays creature info when a creature is selected', async () => {
    const { debug } = render(<App />);
    
    // Wait for initialization
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
    });

    // Get simulation instance and verify it exists
    const simulation = await vi.mocked(initializeSimulation).mock.results[0].value;
    expect(simulation).toBeTruthy();

    // Create mock creature
    const mockCreature = {
      id: '123e4567-e89b-12d3-a456-426614174000',
      age: 10,
      energy: 100,
      generation: 2,
      position: { x: 0, y: 0 },
      velocity: { x: 0, y: 0 },
      rotation: 0,
      fitness: 50,
      children: 0,
      neuralNetwork: {
        inputSize: 8,
        outputSize: 3,
        hiddenLayers: [12, 12],
      }
    };

    // Get the callback that was registered
    const registeredCallback = simulation.setSelectedCreatureCallback.mock.calls[0][0];
    expect(registeredCallback).toBeDefined();

    // Store original console.log for debugging
    const originalConsoleLog = console.log;

    try {
      // Add a debugging console.log
      console.log = (...args) => {
        originalConsoleLog(...args);
      };

      // Simulate requestAnimationFrame for proper callback handling
      const originalRAF = window.requestAnimationFrame;
      window.requestAnimationFrame = (callback) => { 
        callback(0);
        return 0;
      };

      // Call the callback directly within an act block 
      await act(async () => {
        // Invoke the callback which should update state and trigger re-renders
        await registeredCallback(mockCreature);
        
        // Additional wait to ensure state updates are processed
        await new Promise(resolve => setTimeout(resolve, 0));
      });

      // Restore original requestAnimationFrame
      window.requestAnimationFrame = originalRAF;

      // Debugging the component state
      console.log('Current UI after callback:', screen.queryByTestId('ui-container') ? 'UI container exists' : 'No UI container');
      debug();

      // Use a more comprehensive waitFor
      await waitFor(() => {
        const ui = screen.getByTestId('ui-container');
        // Check if the creature info is a child of the UI container
        const creatureInfo = ui.querySelector('[data-testid="creature-info"]');
        expect(creatureInfo).not.toBeNull();
      }, { timeout: 2000 });

      // Now we can safely get the creature info element
      const creatureInfo = screen.getByTestId('creature-info');
      expect(creatureInfo).toBeInTheDocument();

      // Check for content within the creature info component
      expect(within(creatureInfo).getByText(/Selected Creature/i)).toBeInTheDocument();
      
      // Get all paragraphs within the creature info component
      const paragraphs = creatureInfo.querySelectorAll('p');
      const paragraphsArray = Array.from(paragraphs);
      
      // Look for the specific paragraph with Generation information
      const generationParagraph = paragraphsArray.find(p => 
        p.textContent && p.textContent.includes('Generation') && p.textContent.includes('2')
      );
      expect(generationParagraph).toBeTruthy();
      
      // Look for the specific paragraph with Energy information
      const energyParagraph = paragraphsArray.find(p => 
        p.textContent && p.textContent.includes('Energy') && p.textContent.includes('100')
      );
      expect(energyParagraph).toBeTruthy();
    } finally {
      // Restore console.log
      console.log = originalConsoleLog;
    }
  });

  test('handles simulation initialization error', async () => {
    vi.mocked(initializeSimulation).mockRejectedValueOnce(new Error('Initialization failed'));
    render(<App />);
    
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
    });

    expect(screen.getByText(/Error: Initialization failed/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
  });

  test('updates simulation parameters', async () => {
    render(<App />);
    
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
    });

    const mutationRateSlider = screen.getByLabelText(/mutation rate/i);
    await act(async () => {
      fireEvent.change(mutationRateSlider, { target: { value: '0.1' } });
    });

    const foodSpawnRateSlider = screen.getByLabelText(/food spawn rate/i);
    await act(async () => {
      fireEvent.change(foodSpawnRateSlider, { target: { value: '0.8' } });
    });

    expect(mutationRateSlider).toHaveValue('0.1');
    expect(foodSpawnRateSlider).toHaveValue('0.8');
  });
});