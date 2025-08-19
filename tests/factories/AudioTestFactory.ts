/**
 * Audio Test Data Factory
 * 
 * This factory creates realistic test audio data for all test scenarios.
 * All functions create data that will be used to test against implementations
 * that DO NOT YET EXIST - this is pure TDD.
 */

export interface AudioData {
  sampleRate: number;
  channels: 1 | 2;
  samples: Float32Array;
  timestamp: number;
  sourceChannel: 'microphone' | 'system' | 'mixed' | 'unknown';
  durationSeconds: number;
}

export interface ComplexAudioScenario {
  audio: AudioData;
  groundTruth: {
    text: string;
    speakers: Array<{
      startTime: number;
      endTime: number;
      speakerId: string;
      text: string;
    }>;
    languages: string[];
  };
  segments?: any[];
  speakers?: any[];
  complexity?: 'low' | 'medium' | 'high' | 'extreme';
}

export class AudioTestFactory {
  /**
   * Creates clean speech audio for accuracy testing
   * This will be used to test ASR engines that don't exist yet
   */
  static createCleanSpeech(durationSeconds: number): AudioData {
    const sampleRate = 16000;
    const samples = this.generateCleanSpeechSamples(durationSeconds, sampleRate);
    
    return {
      sampleRate,
      channels: 1,
      samples,
      timestamp: Date.now(),
      sourceChannel: 'microphone',
      durationSeconds
    };
  }

  /**
   * Creates realistic business meeting scenario for integration testing
   * Tests that don't exist yet will validate speaker diarization and accuracy
   */
  static createBusinessMeeting(durationSeconds: number): ComplexAudioScenario {
    const speakers = [
      { name: 'Manager', voiceProfile: 'male-us-business', speechRatio: 0.4 },
      { name: 'Developer', voiceProfile: 'female-us-technical', speechRatio: 0.35 },
      { name: 'Designer', voiceProfile: 'male-uk-creative', speechRatio: 0.25 }
    ];
    
    const segments = this.generateMeetingSegments(durationSeconds, speakers);
    
    return {
      audio: this.mixSegmentsToAudio(segments),
      groundTruth: {
        text: segments.map(s => s.text).join(' '),
        speakers: segments.map(s => ({
          startTime: s.startTime,
          endTime: s.endTime,
          speakerId: s.speakerId,
          text: s.text
        })),
        languages: ['en']
      },
      segments,
      speakers,
      complexity: 'medium'
    };
  }

  /**
   * Creates multilingual meeting for language detection testing
   * Future tests will validate language switching and accuracy
   */
  static createMultilingualMeeting(): ComplexAudioScenario {
    const segments = [
      { 
        text: "Good morning everyone, let's start our quarterly review.", 
        language: 'en', 
        speaker: 'speaker1', 
        startTime: 0, 
        endTime: 4 
      },
      { 
        text: "おはようございます。今四半期の業績について話し合いましょう。", 
        language: 'ja', 
        speaker: 'speaker2', 
        startTime: 4, 
        endTime: 9 
      },
      { 
        text: "Thank you. Our revenue this quarter exceeded expectations.", 
        language: 'en', 
        speaker: 'speaker1', 
        startTime: 9, 
        endTime: 13 
      },
      { 
        text: "素晴らしいニュースです。具体的な数字を教えていただけますか？", 
        language: 'ja', 
        speaker: 'speaker2', 
        startTime: 13, 
        endTime: 18 
      },
    ];
    
    return this.createScenarioFromSegments(segments);
  }

  /**
   * Creates noisy conference call for robustness testing
   * Tests will validate noise handling capabilities of non-existent VAD system
   */
  static createNoisyConferenceCall(): AudioData {
    const cleanSpeech = this.createCleanSpeech(60);
    const backgroundNoise = this.generateBackgroundNoise(-25, 60); // -25dB
    const mixedSamples = this.mixAudio(cleanSpeech.samples, backgroundNoise);
    
    return {
      ...cleanSpeech,
      samples: mixedSamples,
      sourceChannel: 'system'
    };
  }

  /**
   * Creates extreme stress test scenario for system limits testing
   * Future performance tests will validate thermal and resource management
   */
  static createStressTestScenario(): ComplexAudioScenario {
    return {
      audio: this.createHighLoadAudio({
        speakers: 8,
        languages: ['en', 'ja', 'es'],
        backgroundNoise: -20,
        crossTalk: true,
        duration: 3600, // 1 hour
        technicalTerms: true
      }),
      groundTruth: this.generateStressTestGroundTruth(),
      complexity: 'extreme'
    };
  }

  /**
   * Creates silence for VAD negative testing
   * VAD tests that don't exist will validate silence rejection
   */
  static createSilence(durationSeconds: number): AudioData {
    const sampleRate = 16000;
    const samples = new Float32Array(durationSeconds * sampleRate);
    // Fill with very low-level noise to simulate microphone noise floor
    for (let i = 0; i < samples.length; i++) {
      samples[i] = (Math.random() - 0.5) * 0.001; // -60dB noise floor
    }
    
    return {
      sampleRate,
      channels: 1,
      samples,
      timestamp: Date.now(),
      sourceChannel: 'microphone',
      durationSeconds
    };
  }

  /**
   * Creates speech with varying noise levels for VAD adaptation testing
   */
  static createSpeechWithNoise(noiseLevel: number): AudioData {
    const cleanSpeech = this.createCleanSpeech(10);
    const noise = this.generateBackgroundNoise(noiseLevel, 10);
    const mixedSamples = this.mixAudio(cleanSpeech.samples, noise);
    
    return {
      ...cleanSpeech,
      samples: mixedSamples
    };
  }

  /**
   * Creates streaming audio chunks for real-time processing tests
   */
  static createStreamingChunks(chunkCount: number): AudioData[] {
    const chunks: AudioData[] = [];
    const chunkDuration = 0.5; // 500ms chunks
    
    for (let i = 0; i < chunkCount; i++) {
      chunks.push({
        sampleRate: 16000,
        channels: 1,
        samples: this.generateCleanSpeechSamples(chunkDuration, 16000),
        timestamp: Date.now() + (i * chunkDuration * 1000),
        sourceChannel: 'microphone',
        durationSeconds: chunkDuration
      });
    }
    
    return chunks;
  }

  // Private helper methods for audio generation

  private static generateCleanSpeechSamples(duration: number, sampleRate: number): Float32Array {
    // Generate synthetic speech-like audio with formant frequencies
    const samples = new Float32Array(duration * sampleRate);
    const fundamental = 150; // Hz
    const formants = [800, 1200, 2400]; // Hz
    
    for (let i = 0; i < samples.length; i++) {
      const t = i / sampleRate;
      let sample = 0;
      
      // Fundamental frequency with formant emphasis
      for (const formant of formants) {
        sample += 0.3 * Math.sin(2 * Math.PI * formant * t) * Math.exp(-t * 0.1);
      }
      
      // Add speech envelope
      const envelope = this.speechEnvelope(t, duration);
      samples[i] = sample * envelope * 0.3;
    }
    
    return samples;
  }

  private static speechEnvelope(t: number, duration: number): number {
    // Simulate natural speech patterns with pauses
    const speechRate = 2.5; // syllables per second
    const syllableTime = t * speechRate;
    const syllablePhase = syllableTime % 1;
    
    // Speech vs silence pattern
    const isSpeech = syllablePhase < 0.6 && (Math.floor(t * 0.5) % 3) < 2;
    
    return isSpeech ? Math.sin(Math.PI * syllablePhase / 0.6) : 0;
  }

  private static generateBackgroundNoise(dbLevel: number, duration: number): Float32Array {
    const sampleRate = 16000;
    const samples = new Float32Array(duration * sampleRate);
    const amplitude = Math.pow(10, dbLevel / 20);
    
    for (let i = 0; i < samples.length; i++) {
      samples[i] = amplitude * (Math.random() * 2 - 1);
    }
    
    return samples;
  }

  private static mixAudio(signal1: Float32Array, signal2: Float32Array): Float32Array {
    const length = Math.min(signal1.length, signal2.length);
    const mixed = new Float32Array(length);
    
    for (let i = 0; i < length; i++) {
      mixed[i] = Math.tanh(signal1[i] + signal2[i]); // Soft clipping
    }
    
    return mixed;
  }

  private static generateMeetingSegments(duration: number, speakers: any[]): any[] {
    // Generate realistic meeting segments with speaker turns
    const segments = [];
    const segmentLength = 5; // Average 5 seconds per segment
    const segmentCount = Math.floor(duration / segmentLength);
    
    const meetingPhrases = [
      "Let's begin today's meeting.",
      "Thank you for joining us.",
      "Our quarterly results show improvement.",
      "I'd like to discuss the technical implementation.",
      "The design approach looks promising.",
      "We should consider the user experience.",
      "What are your thoughts on this proposal?",
      "I agree with that assessment.",
      "Let's move on to the next agenda item.",
      "Any questions before we proceed?"
    ];
    
    for (let i = 0; i < segmentCount; i++) {
      const speaker = speakers[i % speakers.length];
      const startTime = i * segmentLength;
      const endTime = startTime + segmentLength;
      const text = meetingPhrases[i % meetingPhrases.length];
      
      segments.push({
        startTime,
        endTime,
        text,
        speakerId: `speaker_${i % speakers.length + 1}`,
        speaker: speaker.name
      });
    }
    
    return segments;
  }

  private static mixSegmentsToAudio(segments: any[]): AudioData {
    // Mix multiple speaker segments into single audio stream
    const totalDuration = segments[segments.length - 1]?.endTime || 60;
    const sampleRate = 16000;
    const totalSamples = Math.floor(totalDuration * sampleRate);
    const mixedSamples = new Float32Array(totalSamples);
    
    for (const segment of segments) {
      const segmentAudio = this.generateCleanSpeechSamples(
        segment.endTime - segment.startTime, 
        sampleRate
      );
      
      const startIndex = Math.floor(segment.startTime * sampleRate);
      for (let i = 0; i < segmentAudio.length && (startIndex + i) < totalSamples; i++) {
        mixedSamples[startIndex + i] += segmentAudio[i];
      }
    }
    
    return {
      sampleRate,
      channels: 1,
      samples: mixedSamples,
      timestamp: Date.now(),
      sourceChannel: 'mixed',
      durationSeconds: totalDuration
    };
  }

  private static createScenarioFromSegments(segments: any[]): ComplexAudioScenario {
    const audio = this.mixSegmentsToAudio(segments);
    
    return {
      audio,
      groundTruth: {
        text: segments.map(s => s.text).join(' '),
        speakers: segments.map(s => ({
          startTime: s.startTime,
          endTime: s.endTime,
          speakerId: s.speaker,
          text: s.text
        })),
        languages: [...new Set(segments.map(s => s.language || 'en'))]
      },
      complexity: 'high'
    };
  }

  private static createHighLoadAudio(params: any): AudioData {
    // Generate complex audio scenario for stress testing
    const duration = params.duration || 3600;
    const audio = this.createCleanSpeech(duration);
    
    if (params.backgroundNoise) {
      const noise = this.generateBackgroundNoise(params.backgroundNoise, duration);
      audio.samples = this.mixAudio(audio.samples, noise);
    }
    
    return audio;
  }

  private static generateStressTestGroundTruth(): any {
    return {
      text: "This is a complex stress test scenario with multiple speakers and languages.",
      speakers: Array.from({ length: 8 }, (_, i) => ({
        startTime: i * 450, // Each speaker gets ~7.5 minutes
        endTime: (i + 1) * 450,
        speakerId: `speaker_${i + 1}`,
        text: `Speaker ${i + 1} complex technical discussion.`
      })),
      languages: ['en', 'ja', 'es']
    };
  }
}

// Performance test data
export const PerformanceTestScenarios = {
  realTimeLatency: {
    description: 'First word appears within 1.5s of speech start',
    target: 1500, // milliseconds
    testAudio: () => AudioTestFactory.createCleanSpeech(10),
    measure: 'time_to_first_word'
  },
  
  continuousProcessing: {
    description: 'Process 4-hour meeting without degradation',
    target: { rtf: 1.0, memoryLeak: 0 },
    testAudio: () => AudioTestFactory.createBusinessMeeting(14400), // 4 hours
    measure: ['average_rtf', 'memory_stability']
  },
  
  cleanSpeechAccuracy: {
    description: 'Clean English business meeting accuracy',
    target: { wer: 0.10, confidence: 0.90 },
    testAudio: () => AudioTestFactory.createBusinessMeeting(1800), // 30 min
    measure: ['word_error_rate', 'average_confidence']
  },
  
  thermalStability: {
    description: 'Maintain performance under thermal pressure',
    target: { gracefulDegradation: true, noDataLoss: true },
    testAudio: () => AudioTestFactory.createStressTestScenario(),
    measure: ['thermal_response', 'data_integrity']
  }
};