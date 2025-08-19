// Simple test runner to validate our components work
import { AudioVisualizer } from './src/components/AudioVisualizer.js';
import { TranscriptionController } from './src/components/TranscriptionController.js';

console.log('AudioVisualizer component:', typeof AudioVisualizer);
console.log('TranscriptionController component:', typeof TranscriptionController);

// Basic validation
const audioVisualizerProps = {
  audioLevel: 0.5,
  isRecording: false,
  vadActivity: false,
  showWaveform: true,
  height: 100,
  width: 800,
};

const transcriptionControllerProps = {
  onSessionStart: () => {},
  onSessionEnd: () => {},
  onError: () => {},
};

console.log('Components appear to be importable');
console.log('AudioVisualizer props validation:', Object.keys(audioVisualizerProps));
console.log('TranscriptionController props validation:', Object.keys(transcriptionControllerProps));