import React, { useState } from 'react';

interface ControlsPanelProps {
  isPaused: boolean;
  onTogglePause: () => void;
  onReset: () => void;
  mutationRate: number;
  foodSpawnRate: number;
  onMutationRateChange: (value: number) => void;
  onFoodSpawnRateChange: (value: number) => void;
}

const ControlsPanel: React.FC<ControlsPanelProps> = ({
  isPaused,
  onTogglePause,
  onReset,
  mutationRate,
  foodSpawnRate,
  onMutationRateChange,
  onFoodSpawnRateChange,
}) => {
  const [showControls, setShowControls] = useState(true);

  const handleMutationRateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    onMutationRateChange(value);
  };

  const handleFoodSpawnRateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    onFoodSpawnRateChange(value);
  };

  return (
    <div className="controls-panel">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h3 style={{ margin: 0 }}>Controls</h3>
        <button onClick={() => setShowControls(!showControls)}>
          {showControls ? 'Hide' : 'Show'}
        </button>
      </div>
      
      {showControls && (
        <div>
          <div style={{ marginTop: '10px' }}>
            <button onClick={onTogglePause}>
              {isPaused ? 'Resume Simulation' : 'Pause Simulation'}
            </button>
            <button onClick={onReset}>Reset Simulation</button>
          </div>

          <div className="slider-container">
            <label>
              Mutation Rate: {mutationRate.toFixed(2)}
              <input
                type="range"
                min="0"
                max="0.5"
                step="0.01"
                value={mutationRate}
                onChange={handleMutationRateChange}
              />
            </label>
          </div>

          <div className="slider-container">
            <label>
              Food Spawn Rate: {foodSpawnRate.toFixed(2)}
              <input
                type="range"
                min="0.1"
                max="2"
                step="0.1"
                value={foodSpawnRate}
                onChange={handleFoodSpawnRateChange}
              />
            </label>
          </div>

          <div style={{ marginTop: '10px', fontSize: '0.8rem' }}>
            <p>
              <strong>Controls:</strong><br />
              Click: Select creature<br />
              Right-click: Deselect<br />
              Space: Pause/Resume<br />
              R: Reset view<br />
              Mouse wheel/Pinch: Zoom in/out<br />
              Drag: Pan view
            </p>
          </div>
        </div>
      )}
    </div>
  );
};

export default ControlsPanel;