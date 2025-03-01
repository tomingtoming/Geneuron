# Geneuron

[![CI](https://github.com/tomingtoming/geneuron/actions/workflows/ci.yml/badge.svg)](https://github.com/tomingtoming/geneuron/actions/workflows/ci.yml)

Geneuron is an interactive neural evolution simulation where digital creatures evolve and adapt to their environment using neural networks. Watch as creatures learn to find food, avoid predators, and develop complex behaviors through natural selection.

## Features

- 🧠 Neural network-driven creature behavior
- 🧬 Genetic algorithms for evolution and natural selection
- 🌐 Interactive 3D environment with WebGL/Three.js
- 🔄 Real-time simulation with adjustable parameters
- 📊 Statistics and visualization tools
- 🔍 Creature inspection and neural network visualization

## Live Demo

Try out the simulation: [Geneuron Demo](https://geneuron-demo.vercel.app) (Coming soon)

## Getting Started

### Prerequisites

- Node.js (v16.0.0 or higher)
- npm or yarn

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/geneuron.git
   cd geneuron
   ```

2. Install dependencies:
   ```bash
   npm install
   # or
   yarn
   ```

3. Start the development server:
   ```bash
   npm run dev
   # or
   yarn dev
   ```

4. Open your browser and navigate to `http://localhost:3000`

## How to Use

### Controls

- **Click**: Select a creature to inspect
- **Right-click**: Deselect creature
- **Space**: Pause/resume simulation
- **R**: Reset camera view
- **Mouse wheel**: Zoom in/out
- **Shift+drag**: Pan camera

### Simulation Parameters

- **Mutation Rate**: Controls how much creatures' neural networks mutate between generations
- **Food Spawn Rate**: Controls how quickly food appears in the environment

## Technology Stack

- **TypeScript**: Type-safe programming
- **Three.js**: 3D visualization and rendering
- **TensorFlow.js**: Neural network implementation
- **React**: User interface components
- **Vite**: Fast development environment

## Architecture

The project follows a modular architecture:

```
geneuron/
  ├── src/
  │   ├── components/     # UI components
  │   ├── core/           # Core simulation logic
  │   │   ├── neural/     # Neural network implementation
  │   │   ├── physics/    # Physics simulation
  │   │   ├── creature/   # Creature behavior and genetics
  │   │   ├── food/       # Food resources
  │   │   └── world/      # World management and simulation
  │   ├── rendering/      # Three.js rendering
  │   └── utils/          # Utility functions
```

## Project Roadmap

- [x] Basic simulation environment
- [x] Neural network implementation
- [x] Creature behavior and reproduction
- [x] Food spawning and consumption
- [ ] Predator-prey relationships
- [ ] Enhanced visualization tools
- [ ] Exportable/importable neural networks
- [ ] Custom scenarios and environments

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Inspired by classic artificial life simulations
- Built using modern web technologies for accessibility
- Special thanks to the TensorFlow.js and Three.js communities