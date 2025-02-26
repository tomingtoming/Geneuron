# Geneuron Development Guidelines

This document outlines the development practices and coding standards for the Geneuron project.

## Rust Development Best Practices

When working with Rust code in this project, follow these guidelines:

### API Usage
1. Use complete path qualification for external crates to avoid conflicts:
   ```rust
   use ::rand::Rng;
   use ::rand::prelude::IteratorRandom;
   ```

2. Prefer modern API methods over deprecated ones:
   - Use `random::<T>()` instead of `gen()`
   - Use `random_range()` instead of `gen_range()`
   - Use `random_bool()` instead of `gen_bool()`

3. Handle type conversions explicitly:
   - Use `.into()` for safe type conversions
   - Use explicit type annotations when needed
   - Handle floating-point conversions carefully (f32 to f64)

### Code Organization
1. Dead Code Management:
   - Add `#[allow(dead_code)]` for intentionally unused items
   - Document why code is kept despite being unused
   - Keep potential future use cases in mind

2. Variable Naming:
   - Use `_variable` prefix for intentionally unused variables
   - Use descriptive names that reflect purpose
   - Follow Rust naming conventions

3. Memory Management:
   - Pre-allocate vectors when size is known: `Vec::with_capacity(size)`
   - Minimize cloning, prefer references when possible
   - Use appropriate ownership models

### Error Handling
1. Pattern Matching:
   - Use `if let` for single pattern matches
   - Use `match` for multiple patterns
   - Handle edge cases explicitly

2. Error Propagation:
   - Use `?` operator for Result types
   - Provide meaningful error messages
   - Consider wrapping external errors

### Performance
1. Collection Management:
   - Use appropriate collection types
   - Pre-allocate when possible
   - Consider using iterators over loops

2. Algorithm Optimization:
   - Profile before optimizing
   - Use efficient data structures
   - Consider space-time tradeoffs

### Documentation
1. Code Comments:
   - Document complex algorithms
   - Explain non-obvious decisions
   - Keep comments up to date
   - Use English consistently for all comments
   - Translate any non-English comments during code review

2. API Documentation:
   - Document public interfaces
   - Include examples
   - Explain panics and errors
   - Use English consistently across all documentation

### Testing
1. Unit Tests:
   - Test edge cases
   - Test error conditions
   - Use appropriate test helpers

2. Integration Tests:
   - Test major features
   - Test interactions between components
   - Simulate real-world scenarios

### Maintenance
1. Keep Dependencies Updated:
   - Review release notes
   - Test thoroughly after updates
   - Follow semver guidelines
   - Keep GitHub Actions versions up to date (use latest stable versions)

### Project-Specific Guidelines
1. Simulation Parameters:
   - Use constants for magic numbers
   - Document parameter effects
   - Consider configuration options

2. Physics Calculations:
   - Use appropriate floating-point types
   - Handle edge cases
   - Document assumptions

3. Neural Network:
   - Document network architecture
   - Handle numerical stability
   - Consider optimization techniques

4. Creature Behavior:
   - Document state transitions
   - Consider energy balance
   - Test edge cases

## Recent Development Context

### Camera Management
- Camera position has constraints to prevent the world from disappearing when zoomed out
- Added the `constrain_camera()` function to enforce these boundaries
- Added a `reset_view()` function triggered by the R key
- Camera follows selected creatures when in follow mode (toggle with F key)

### User Interface
- Zoom controls: Z/X keys and mouse wheel
- Pause simulation: Space key
- Select creatures: Left click
- Deselect: Right click
- Move camera: Shift+drag or middle mouse button
- Reset view: R key
- Follow selected creature: F key

### Visual Feedback
- Selected creatures highlighted in yellow
- Hover effect for creatures under cursor
- Energy levels displayed as colored rings
- Grid system for spatial reference
- Status information displayed in top-left corner
- Controls help displayed in bottom-left corner
- Detailed creature information displayed when selected

### World Wrapping
- The simulation world is toroidal (wraps around edges)
- Drawing functions account for this with wrapped rendering
- Camera movement considers shortest paths in wrapped space