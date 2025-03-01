import React, { useState } from 'react';

interface NeuralNetworkInfo {
  inputSize: number;
  outputSize: number;
  hiddenLayers: number[];
}

interface CreatureInfoProps {
  creature: {
    id: string;
    age: number;
    energy: number;
    generation: number;
    neuralNetwork: NeuralNetworkInfo;
    position: { x: number; y: number };
    velocity: { x: number; y: number };
    rotation: number;
    fitness: number;
    children: number;
  };
}

const CreatureInfo: React.FC<CreatureInfoProps> = ({ creature }) => {
  const [showDetails, setShowDetails] = useState(false);

  // Format position and velocity to 2 decimal places
  const formatNumber = (num: number) => Math.round(num * 100) / 100;

  return (
    <div className="creature-info">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h3>Selected Creature</h3>
        <button onClick={() => setShowDetails(!showDetails)}>
          {showDetails ? 'Less Info' : 'More Info'}
        </button>
      </div>
      
      <div>
        <p><strong>ID:</strong> {creature.id.substring(0, 8)}...</p>
        <p><strong>Generation:</strong> {creature.generation}</p>
        <p><strong>Age:</strong> {formatNumber(creature.age)}</p>
        <p><strong>Energy:</strong> {formatNumber(creature.energy)}</p>
        <p><strong>Fitness:</strong> {formatNumber(creature.fitness)}</p>
        <p><strong>Children:</strong> {creature.children}</p>
        
        {showDetails && (
          <>
            <h4>Position & Movement</h4>
            <p><strong>Position:</strong> ({formatNumber(creature.position.x)}, {formatNumber(creature.position.y)})</p>
            <p><strong>Velocity:</strong> ({formatNumber(creature.velocity.x)}, {formatNumber(creature.velocity.y)})</p>
            <p><strong>Speed:</strong> {formatNumber(Math.sqrt(creature.velocity.x * creature.velocity.x + creature.velocity.y * creature.velocity.y))}</p>
            <p><strong>Rotation:</strong> {formatNumber(creature.rotation)} rad</p>
            
            <h4>Neural Network</h4>
            <p><strong>Inputs:</strong> {creature.neuralNetwork.inputSize}</p>
            <p><strong>Hidden Layers:</strong> [{creature.neuralNetwork.hiddenLayers.join(', ')}]</p>
            <p><strong>Outputs:</strong> {creature.neuralNetwork.outputSize}</p>
          </>
        )}
      </div>
    </div>
  );
};

export default CreatureInfo;