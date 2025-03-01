import { render, screen, fireEvent, within } from '@testing-library/react';
import { vi, describe, test, expect } from 'vitest';
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

    // Call the callback in an act block
    await act(async () => {
      await registeredCallback(mockCreature);
    });

    // Wait for state updates to propagate
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
    });

    // Debug current DOM state
    screen.debug();

    // Wait for the creature info to appear
    const uiContainer = await screen.findByTestId('ui-container');
    expect(uiContainer).toBeInTheDocument();

    // Now look for the creature info
    const creatureInfo = await screen.findByTestId('creature-info', {}, { timeout: 2000 });
    expect(creatureInfo).toBeInTheDocument();

    // Verify the content
    expect(within(creatureInfo).getByText(/Selected Creature/i)).toBeInTheDocument();
    expect(within(creatureInfo).getByText(/Generation: 2/)).toBeInTheDocument();
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