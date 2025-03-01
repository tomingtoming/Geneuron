import React from 'react';
import ReactDOM from 'react-dom/client';
import * as tf from '@tensorflow/tfjs';
import App from './App';
import './styles.css';

// Initialize TensorFlow.js before rendering
(async () => {
  try {
    console.log('Initializing TensorFlow.js...');
    await tf.ready();
    console.log('TensorFlow.js initialized with backend:', tf.getBackend());
    
    // Render the app after TensorFlow.js is ready
    ReactDOM.createRoot(document.getElementById('root')!).render(
      <React.StrictMode>
        <App />
      </React.StrictMode>
    );
  } catch (error) {
    console.error('Failed to initialize TensorFlow.js:', error);
    // Show error message to user
    document.body.innerHTML = '<div style="color: red; padding: 20px;">Failed to initialize TensorFlow.js. Please check your browser console for details.</div>';
  }
})();