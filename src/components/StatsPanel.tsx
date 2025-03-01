import React from 'react';

interface StatsPanelProps {
  stats: {
    fps: number;
    creatureCount: number;
    foodCount: number;
    generation: number;
    elapsedTime: number;
  };
}

const StatsPanel: React.FC<StatsPanelProps> = ({ stats }) => {
  // Format elapsed time as minutes:seconds
  const formatElapsedTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  };

  return (
    <div className="stats-panel">
      <h3>Simulation Stats</h3>
      <div>
        <p><strong>FPS:</strong> {stats.fps}</p>
        <p><strong>Creatures:</strong> {stats.creatureCount}</p>
        <p><strong>Food:</strong> {stats.foodCount}</p>
        <p><strong>Generation:</strong> {stats.generation}</p>
        <p><strong>Elapsed Time:</strong> {formatElapsedTime(stats.elapsedTime)}</p>
      </div>
    </div>
  );
};

export default StatsPanel;