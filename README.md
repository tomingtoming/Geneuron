# Geneuron

Geneuron is a neural evolution simulation where creatures evolve over time to adapt to their environment. The simulation uses neural networks to control the behavior of the creatures, and they can reproduce, feed, and interact with each other.

## Features

- Neural network-based behavior
- Reproduction and mutation
- Energy management
- Group behavior
- Detailed creature information display
- Smooth zoom and camera control

## Requirements

- Rust 1.70.0 or higher
- Graphics driver supporting OpenGL 3.2 or higher

## Installation

1. Clone the repository:

```sh
$ git clone https://github.com/tomingtoming/geneuron.git
```

2. Navigate to the project directory:

```sh
$ cd geneuron
```

3. Build and run the project:

```sh
$ cargo run
```

## Controls

- **Space**: Pause/Unpause the simulation
- **Z**: Zoom in
- **X**: Zoom out
- **F**: Toggle follow mode for selected creature
- **Left Click**: Select a creature
- **Right Click**: Deselect the selected creature

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

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Author

tomingtoming

## Contributing

Issues and pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.