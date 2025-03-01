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
   - Mock requestAnimationFrame when dealing with animation or render timing
   - Be aware that some state updates might require multiple render cycles
   - Consider using waitFor with appropriate assertions for complex state changes

2. DOM Element Selection
   - Prefer more specific queries over general ones
   - When dealing with text content split across elements, use custom matcher functions
   - Use within() to scope queries to specific components
   - Consider the component hierarchy when selecting elements
   - Use getAllBy* when multiple elements might match and you need to filter

3. Simulation Mocks
   - Mock the entire simulation lifecycle
   - Consider both synchronous and asynchronous callbacks
   - Ensure mocks match the real implementation's behavior
   - Validate callback registration and execution
   - Add appropriate logging in mocks for debugging
   - Ensure mock callbacks return Promises when the real implementation does
   - Consider the timing of callback registration and cleanup
   - Mock complex state management (e.g., requestAnimationFrame, setTimeout)
   - Add appropriate logging points for debugging state updates
   - Implement proper cleanup in mock teardown

### Test Isolation
1. State Management
   - Reset all mocks between tests using beforeEach
   - Clean up subscriptions and event listeners
   - Reset any global state that might affect other tests
   - Consider using separate describe blocks for related test cases
   - Handle test-specific setup and teardown properly

2. Test Independence
   - Avoid sharing mutable state between tests
   - Create fresh mock instances for each test
   - Reset DOM state between tests
   - Clean up any side effects from previous tests
   - Consider test order independence

### Debugging Test Failures
1. Structured Approach
   - Add strategic console.logs for state changes
   - Use screen.debug() at key points to inspect DOM
   - Check component lifecycle timing
   - Verify mock function calls and their timing
   - Log state updates and their effects

2. Common Pitfalls
   - Text content split across multiple elements
   - Async state updates not properly waited for
   - Race conditions in component mounting
   - Incomplete mock implementations
   - Missing cleanup in test teardown
   - Incorrect component hierarchy assumptions

### Test Utilities
1. Custom Matchers
   ```typescript
   // Example: Text content matcher across elements
   const getByTextContent = (container: HTMLElement, text: string) => {
     const elements = container.querySelectorAll('*');
     return Array.from(elements).find(element => 
       element.textContent?.includes(text)
     );
   };
   ```

2. Helper Functions
   ```typescript
   // Example: Wait for multiple state updates
   const waitForUpdates = async (timeout = 100) => {
     await act(async () => {
       await new Promise(resolve => setTimeout(resolve, timeout));
     });
   };
   ```

### Best Practices for Complex Components
1. Component Testing
   - Test the complete lifecycle
   - Verify all possible states
   - Check error boundaries
   - Test component interactions
   - Validate cleanup on unmount
   - Test with different prop combinations

2. Integration Testing
   - Test component interactions
   - Verify data flow between components
   - Test state synchronization
   - Handle edge cases in integration
   - Verify error propagation

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
- Consider text content that might be split across elements
- Use appropriate element queries based on context
- Implement thorough cleanup in tests
- Add strategic debug points
- Handle all possible component states
- Test error cases explicitly

# Copilot Instructions for Geneuron Development

This document outlines preferences and best practices for developing the Geneuron project, based on our learnings from development experience.

## Project-Specific Guidelines

### Implementation Strategy

1. Initial Phase:
   - Start with core simulation logic first
   - Focus on type safety from the beginning
   - Implement basic functionality before adding advanced features
   - Use proper abstraction layers for easy refactoring

2. Refactoring Approach:
   - Make changes incrementally
   - Remove deprecated code immediately to prevent confusion
   - Keep related changes in separate commits
   - Document major architectural decisions

### Code Organization

1. Directory Structure:
   ```
   src/
     ├── components/     # React components
     ├── core/          # Core simulation logic
     │   ├── neural/    # Neural network implementation
     │   ├── physics/   # Physics simulation
     │   ├── creature/  # Creature behavior
     │   ├── food/      # Food resources
     │   └── world/     # World management
     ├── rendering/     # Three.js visualization
     └── utils/         # Shared utilities
   ```

2. Module Dependencies:
   - Maintain clear dependency boundaries
   - Avoid circular dependencies
   - Use dependency injection for testing
   - Keep implementation details private

### Code Style

1. TypeScript Best Practices:
   ```typescript
   // Good: Strong typing with clear interfaces
   interface SimulationConfig {
     mutationRate: number;
     foodSpawnRate: number;
     worldSize: number;
   }

   // Good: Proper error handling with type safety
   function initializeSimulation(config: SimulationConfig): Promise<Simulation> {
     try {
       // Implementation
     } catch (error) {
       throw new SimulationError('Failed to initialize', { cause: error });
     }
   }
   ```

2. React Component Style:
   ```typescript
   // Good: Functional components with proper types
   interface ControlsPanelProps {
     isPaused: boolean;
     onTogglePause: () => void;
     mutationRate: number;
     onMutationRateChange: (rate: number) => void;
   }

   const ControlsPanel: React.FC<ControlsPanelProps> = ({
     isPaused,
     onTogglePause,
     mutationRate,
     onMutationRateChange
   }) => {
     // Implementation
   };
   ```

### Testing Strategy

1. Test Organization:
   ```typescript
   describe('Simulation', () => {
     // Good: Proper setup and cleanup
     beforeEach(() => {
       vi.useFakeTimers();
     });

     afterEach(() => {
       vi.useRealTimers();
       cleanup();
     });

     // Good: Clear test cases with proper async handling
     test('initializes with correct configuration', async () => {
       await act(async () => {
         // Test implementation
       });
     });
   });
   ```

2. Mock Implementation:
   ```typescript
   // Good: Proper type safety in mocks
   const mockSimulation = vi.fn<[], Promise<Simulation>>()
     .mockImplementation(async () => ({
       // Implementation
     }));
   ```

### Continuous Integration

1. GitHub Actions:
   - Run tests on all PRs
   - Include type checking
   - Automate deployment
   - Check code formatting

2. Quality Gates:
   - All tests must pass
   - No TypeScript errors
   - Code coverage requirements
   - Performance benchmarks

## Development Workflow

### Feature Implementation

1. Planning Phase:
   - Break down requirements
   - Define interfaces first
   - Plan test coverage
   - Consider performance implications

2. Implementation Phase:
   - Write tests first
   - Implement core logic
   - Add UI components
   - Document as you go

### Code Review Guidelines

1. Review Checklist:
   - Type safety
   - Error handling
   - Performance considerations
   - Test coverage
   - Documentation
   - Clean code principles

2. Common Pitfalls:
   - Memory leaks in Three.js/TensorFlow.js
   - Async operation handling
   - Resource cleanup
   - Browser compatibility

## Performance Optimization

### Rendering Performance

1. Three.js Best Practices:
   ```typescript
   // Good: Proper resource management
   class Renderer {
     private dispose() {
       // Clean up Three.js resources
       this.geometry.dispose();
       this.material.dispose();
       this.texture.dispose();
     }
   }
   ```

2. React Optimization:
   ```typescript
   // Good: Proper memoization
   const MemoizedComponent = React.memo(({ data }) => {
     const processedData = useMemo(() => processData(data), [data]);
     return <div>{processedData}</div>;
   });
   ```

### Neural Network Optimization

1. TensorFlow.js Best Practices:
   ```typescript
   // Good: Proper tensor management
   const predict = (inputs: number[]): number[] => {
     return tf.tidy(() => {
       const inputTensor = tf.tensor2d([inputs]);
       const output = model.predict(inputTensor);
       return Array.from(output.dataSync());
     });
   };
   ```

## Documentation Standards

### Code Documentation

1. Component Documentation:
   ```typescript
   /**
    * Creature component represents a single entity in the simulation.
    * 
    * @param props - Component properties
    * @param props.position - Initial position in the world
    * @param props.neuralNetwork - Neural network controlling behavior
    * 
    * @example
    * <Creature
    *   position={{ x: 0, y: 0 }}
    *   neuralNetwork={network}
    * />
    */
   ```

2. Algorithm Documentation:
   ```typescript
   /**
    * Updates creature positions based on neural network outputs and physics.
    * 
    * Algorithm:
    * 1. Get sensory inputs
    * 2. Process through neural network
    * 3. Apply outputs to movement
    * 4. Handle collisions
    * 
    * @param creatures - Array of creatures to update
    * @param deltaTime - Time since last update
    */
   ```

## Error Handling

### Error Types

```typescript
// Good: Custom error types for different scenarios
class SimulationError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = 'SimulationError';
  }
}

class NeuralNetworkError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = 'NeuralNetworkError';
  }
}
```

### Error Recovery

```typescript
// Good: Proper error handling with recovery
async function initializeWithRetry(
  maxAttempts = 3,
  delayMs = 1000
): Promise<Simulation> {
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await initializeSimulation();
    } catch (error) {
      if (attempt === maxAttempts) throw error;
      await new Promise(resolve => setTimeout(resolve, delayMs));
    }
  }
  throw new Error('Failed to initialize after maximum attempts');
}
```

## Commit Message Standards

1. Format:
   ```
   type(scope): subject

   body

   footer
   ```

2. Types:
   - feat: New feature
   - fix: Bug fix
   - refactor: Code change that neither fixes a bug nor adds a feature
   - test: Adding missing tests or correcting existing tests
   - docs: Documentation only changes
   - ci: Changes to CI configuration files and scripts
   - perf: A code change that improves performance

3. Examples:
   ```
   feat(neural): add mutation rate adjustment based on fitness

   refactor(physics): optimize collision detection algorithm
   
   fix(rendering): prevent memory leak in Three.js cleanup
   ```

Remember:
- Keep code modular and testable
- Maintain strong type safety
- Clean up resources properly
- Document complex algorithms
- Consider performance implications
- Write meaningful tests
- Use clear commit messages