# Geneuron

[![Build](https://github.com/tomingtoming/geneuron/actions/workflows/build.yml/badge.svg)](https://github.com/tomingtoming/geneuron/actions/workflows/build.yml)

Geneuron is a neural evolution simulation where creatures evolve over time to adapt to their environment. The simulation uses neural networks to control the behavior of the creatures, and they can reproduce, feed, and interact with each other.

## Features

- Neural network-based behavior
- Reproduction and mutation
- Energy management
- Group behavior
- Detailed creature information display
- Smooth zoom and camera control
- Cross-platform support (Windows, macOS, Linux, Web)

## Requirements

- Rust 1.70.0 or higher
- For native builds:
  - Graphics driver supporting OpenGL 3.2 or higher
- For web builds:
  - `wasm32-unknown-unknown` target installed
  - `wasm-server-runner` for development

## Installation

1. Install Rust if you haven't already:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Add WebAssembly target (for web builds):
```sh
rustup target add wasm32-unknown-unknown
```

3. Clone the repository:
```sh
git clone https://github.com/tomingtoming/geneuron.git
cd geneuron
```

## Building and Running

### Desktop (Windows, macOS, Linux)

1. Development build and run:
```sh
cargo run
```

2. Release build:
```sh
cargo build --release
```

The output binary will be in `target/release/`:
- Windows: `geneuron.exe`
- macOS/Linux: `geneuron`

### Web Browser

1. Development mode with hot-reload:
```sh
cargo run --target wasm32-unknown-unknown
```

2. Release build:
```sh
cargo build --target wasm32-unknown-unknown --release
```

To serve the release build:
1. Create a distribution directory:
```sh
mkdir -p dist
cp index.html dist/
cp target/wasm32-unknown-unknown/release/geneuron.wasm dist/
```

2. Serve with any static file server, for example:
```sh
python3 -m http.server --directory dist
```

Then open `http://localhost:8000` in your web browser.

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