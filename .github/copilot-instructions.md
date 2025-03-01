# Geneuron Development Guidelines

This document outlines the development practices and coding standards for the Geneuron project.

## TypeScript Development Best Practices

When working with TypeScript code in this project, follow these guidelines:

### Type Safety

1. Use strict typing:
   - Avoid `any` types whenever possible
   - Define interfaces and types for all data structures
   - Use generics for reusable components and functions
   - Utilize TypeScript's utility types when appropriate (e.g., `Partial<T>`, `Pick<T>`, etc.)

2. Handle null/undefined consistently:
   - Use optional chaining (`?.`) and nullish coalescing (`??`) operators
   - Consider strict null checks in tsconfig
   - Validate function inputs to prevent runtime errors

3. Type Assertions:
   - Prefer type guards (`if (typeof x === 'string')`) over type assertions (`as`)
   - Use `as` only when you're confident about the type and TypeScript cannot infer it
   - Document any unsafe type assertions

### Code Organization

1. Module Structure:
   - Follow single responsibility principle
   - Group related functionality in modules
   - Use barrel exports (`index.ts`) for cleaner imports
   - Keep file sizes manageable (consider splitting if > 300 lines)

2. Variable Naming:
   - Use camelCase for variables and functions
   - Use PascalCase for classes, interfaces, types, and components
   - Use ALL_CAPS for constants
   - Prefix private class properties with `_`

3. Memory Management:
   - Clean up resources in `useEffect` cleanup functions
   - Dispose of Three.js objects when no longer needed
   - Be mindful of closure-related memory leaks
   - Release TensorFlow.js tensors with `tf.dispose()` or `tf.tidy()`

### Error Handling

1. Promise Handling:
   - Use async/await with proper try/catch blocks
   - Consider central error handling for API calls
   - Provide meaningful error messages to users
   - Log detailed errors for debugging

2. Graceful Degradation:
   - Handle edge cases explicitly
   - Provide fallbacks for missing features
   - Check browser compatibility for advanced features

### Performance Optimization

1. Rendering Performance:
   - Use React's memo, useMemo, and useCallback appropriately
   - Implement virtualization for long lists
   - Optimize Three.js rendering with proper techniques
   - Use efficient algorithms for simulation updates

2. Neural Network Optimization:
   - Batch operations with TensorFlow.js
   - Use WebGL backend when available
   - Consider smaller network architectures for real-time updates
   - Use tf.tidy() for automatic tensor cleanup

### Documentation

1. Code Comments:
   - Document complex algorithms
   - Explain non-obvious decisions
   - Document parameters and return types (especially when not obvious from TypeScript)
   - Include examples for complex functions

2. Component Documentation:
   - Document props with JSDoc comments
   - Explain side effects
   - Note any performance considerations

## Project Architecture

The project follows a modular architecture with these major components:

### Core Simulation
- `neural/`: Neural network implementation using TensorFlow.js
- `physics/`: Physics simulation including collision detection and movement
- `creature/`: Creature behavior, reproduction, and genetics
- `food/`: Food resources, spawning, and consumption
- `world/`: World management, simulation loop, and environmental factors

### Visualization
- `rendering/`: Three.js rendering, camera management, and visual effects

### User Interface
- `components/`: React components for UI elements
- `App.tsx`: Main application component that integrates simulation and UI

## Current State of Development

### Completed Features
- Basic simulation environment with Three.js
- Neural network implementation with TensorFlow.js
- Creature behavior driven by neural networks
- Food spawning and consumption mechanics
- Creature reproduction and genetic inheritance
- Basic UI for simulation control and stats display

### In Progress
- Performance optimization for large-scale simulations
- Advanced visualization for neural networks
- Improved creature behavior and sensory inputs
- Enhanced UI with more detailed statistics and controls

### Future Development
- Predator-prey relationships
- Different species with specialized traits
- Environmental challenges and seasonal changes
- User-definable scenarios and configurations
- Exportable/importable neural networks

## Technology Stack Details

### TypeScript
- Version: 5.x
- Configuration: Strict mode enabled
- Key features: Interfaces, generics, utility types

### Three.js
- Used for 3D visualization and rendering
- Custom shaders for visual effects
- Performance optimizations for large numbers of entities

### TensorFlow.js
- Neural network architecture: Multi-layer perceptron
- Uses WebGL acceleration when available
- Implements custom training and mutation algorithms

### React
- Functional components with hooks
- Context API for state management
- Custom hooks for simulation interaction

## Workflow Integration

When working with Copilot:

1. Add detailed JSDoc comments for better suggestions
2. Specify types for function parameters and return values
3. Break complex tasks into smaller functions with clear purpose
4. Request complete implementations with proper error handling and type safety

## Code Style Guidelines

- Use 2-space indentation
- Semicolons at the end of statements
- Single quotes for strings
- Trailing commas in multi-line arrays and objects
- Meaningful variable names that reflect purpose
- Avoid abbreviations unless very common
- Maximum line length: 100 characters

## Testing Strategy

1. Unit Tests:
   - Test core algorithms and utilities
   - Use testing-library for React components
   - Mock dependencies and external services

2. Integration Tests:
   - Test interactions between modules
   - Ensure proper data flow through the system

3. Performance Testing:
   - Monitor frame rates for rendering
   - Test with different simulation sizes
   - Profile memory usage and optimize as needed

# Copilot Instructions for Geneuron Project

## Testing Best Practices

### Asynchronous Testing
1. State Updates
   - Always wrap React state updates in `act()`
   - Give sufficient time for state updates to propagate
   - Consider multiple re-render cycles
   - Use `findBy*` queries instead of `getBy*` for elements that appear asynchronously
   - Add appropriate timeouts for async operations

2. Simulation Mocks
   - Mock the entire simulation lifecycle
   - Consider both synchronous and asynchronous callbacks
   - Ensure mocks match the real implementation's behavior
   - Validate callback registration and execution
   - Add appropriate logging in mocks for debugging

3. Component Testing
   - Test the complete render cycle
   - Verify initial state and subsequent updates
   - Use `data-testid` for reliable element selection
   - Test component hierarchy and parent-child relationships
   - Consider side effects and cleanup

### Test Structure
1. Initialization
   ```typescript
   test('component behavior', async () => {
     // 1. Setup and render
     const { debug } = render(<Component />);
     
     // 2. Wait for initial mounting
     await act(async () => {
       await new Promise(resolve => setTimeout(resolve, 100));
     });
     
     // 3. Verify initial state
     expect(screen.getByTestId('initial-element')).toBeInTheDocument();
     
     // 4. Trigger state changes
     await act(async () => {
       // Perform actions
       await new Promise(resolve => setTimeout(resolve, 0));
     });
     
     // 5. Verify updated state
     const updatedElement = await screen.findByTestId('updated-element');
     expect(updatedElement).toBeInTheDocument();
   });
   ```

2. Error Cases
   - Test error states explicitly
   - Mock error conditions in dependencies
   - Verify error handling and recovery
   - Test boundary conditions

### Test Debugging
1. Debug Strategies
   - Use `screen.debug()` to inspect DOM state
   - Add console.logs in key state changes
   - Verify mock function calls
   - Check component lifecycle timing

2. Common Issues
   - Asynchronous state updates not waiting
   - Mock implementations incomplete
   - React rendering cycle timing
   - Event propagation timing

## Component Development

### State Management
1. Component State
   - Use appropriate state initialization
   - Consider side effects of state updates
   - Handle null/undefined states
   - Implement proper cleanup

2. Props Interface
   - Define strict prop types
   - Document required vs optional props
   - Consider prop validation
   - Use TypeScript utility types

### Testing Considerations
1. Component Design
   - Add data-testid attributes
   - Consider test requirements during development
   - Implement testable interfaces
   - Document testing requirements

2. Debug Support
   - Add meaningful console logs
   - Consider development-only debugging features
   - Document debugging approaches
   - Include error boundaries

## Project Organization

### Test Files
1. Structure
   - Group related tests
   - Separate mock implementations
   - Share test utilities
   - Maintain test data

2. Naming
   - Clear test descriptions
   - Meaningful variable names
   - Consistent file naming
   - Group related tests

### Code Quality
1. Type Safety
   - Use strict typing
   - Define interfaces
   - Document type assumptions
   - Handle edge cases

2. Error Handling
   - Consistent error patterns
   - Meaningful error messages
   - Error recovery strategies
   - Error boundaries

## Simulation Testing

### Mock Implementation
1. Simulation State
   - Mock core simulation features
   - Handle async operations
   - Maintain state consistency
   - Implement cleanup

2. Callbacks
   - Register callbacks properly
   - Handle callback timing
   - Clean up callback references
   - Test callback error cases

### Integration Points
1. Component Integration
   - Test simulation-component interaction
   - Verify data flow
   - Test state synchronization
   - Handle race conditions

2. State Updates
   - Coordinate multiple state updates
   - Handle update timing
   - Verify state consistency
   - Test update sequences

## Performance Considerations

### Test Performance
1. Test Execution
   - Optimize test runtime
   - Minimize unnecessary waits
   - Clean up resources
   - Handle test isolation

2. Debugging Support
   - Add performance markers
   - Track timing issues
   - Monitor resource usage
   - Profile test execution

## Documentation

### Test Documentation
1. Test Cases
   - Document test purpose
   - Explain complex scenarios
   - Document assumptions
   - Include examples

2. Debug Guide
   - Document common issues
   - Include debug steps
   - Add troubleshooting tips
   - Maintain solutions

Remember:
- Always wrap React state updates in `act()`
- Use `findBy*` for async elements
- Implement complete mock lifecycles
- Consider multiple re-render cycles
- Add proper cleanup
- Document debugging approaches