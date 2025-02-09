# Geneuron

A neural evolution simulation program that combines artificial life with neural networks, visually representing how organisms evolve while searching for food.

## Features

- Simple neural network-based organism behavior control
- Real-time physics simulation
- Cross-platform support (Windows, macOS, Linux)
- Interactive visualization
- Smooth movement and natural behavior

## Requirements

- Rust 1.70.0 or higher
- Graphics driver supporting OpenGL 3.2 or higher

## Installation

```bash
git clone git@github.com:tomingtoming/geneuron.git
cd geneuron
cargo build --release
```

## Running the Simulation

```bash
cargo run --release
```

## Controls

- Space key: Pause/Resume simulation
- Z key: Zoom in
- X key: Zoom out

## How the Simulation Works

### Organism Characteristics
- Each organism is controlled by a neural network with 5 inputs (energy, velocity, rotation, distance and angle to nearest food)
- Energy consumption during movement and recovery through food consumption
- Physical inertia and smooth movement implementation

### Environment
- Automatically replenishing food sources
- Closed environment with boundary reflection
- Real-time status display (generation count, organism count, elapsed time, FPS)

## License

MIT License

## Author

tomingtoming

## Contributing

Issues and pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.