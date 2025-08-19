/**
 * Transcription Test Data Factory
 * 
 * Creates realistic transcription data for testing components that don't exist yet.
 * This factory follows TDD principles - all data represents expected outputs
 * from future implementations.
 */

export interface TranscriptionSegment {
  id: string;
  startTime: number;
  endTime: number;
  text: string;
  speakerId?: string;
  language: string;
  confidence: number;
  words: WordTiming[];
  processingPass: 1 | 2;
  createdAt: number;
  updatedAt?: number;
}

export interface WordTiming {
  word: string;
  startTime: number;
  endTime: number;
  confidence: number;
}

export interface SpeakerProfile {
  id: string;
  name?: string;
  embedding: number[];
  languagePreference: string;
  totalSpeechTime: number;
  segmentCount: number;
  createdAt: number;
  lastActive: number;
  isPersistent: boolean;
}

export interface TranscriptionQualityMetrics {
  overallConfidence: number;
  speakerConsistency: number;
  languageDetectionAccuracy: number;
  realTimeFactor: number;
  memoryUsagePeak: number;
  cpuUsageAverage: number;
  thermalEvents: ThermalEvent[];
  processingLatency: {
    pass1Average: number;
    pass2Average: number;
    p95: number;
    p99: number;
  };
}

export interface ThermalEvent {
  timestamp: number;
  temperature: number;
  action: 'throttle' | 'model_downgrade' | 'emergency_stop';
  durationMs: number;
}

export class TranscriptionTestFactory {
  /**
   * Creates basic transcription segments for unit testing
   * These will test components that don't exist yet
   */
  static createBasicTranscriptionSegments(): TranscriptionSegment[] {
    return [
      {
        id: '550e8400-e29b-41d4-a716-446655440001',
        startTime: 0.0,
        endTime: 3.5,
        text: "Good morning everyone, let's begin today's meeting.",
        speakerId: '550e8400-e29b-41d4-a716-446655440010',
        language: 'en',
        confidence: 0.95,
        words: this.createWordTimings("Good morning everyone, let's begin today's meeting."),
        processingPass: 2,
        createdAt: Date.now(),
        updatedAt: Date.now() + 1000
      },
      {
        id: '550e8400-e29b-41d4-a716-446655440002',
        startTime: 4.0,
        endTime: 7.2,
        text: "Thank you for joining us today.",
        speakerId: '550e8400-e29b-41d4-a716-446655440011',
        language: 'en',
        confidence: 0.92,
        words: this.createWordTimings("Thank you for joining us today."),
        processingPass: 1,
        createdAt: Date.now() + 4000
      }
    ];
  }

  /**
   * Creates multilingual segments for language detection testing
   * Tests future language detection and routing capabilities
   */
  static createMultilingualSegments(): TranscriptionSegment[] {
    return [
      {
        id: '550e8400-e29b-41d4-a716-446655440003',
        startTime: 0.0,
        endTime: 4.0,
        text: "Welcome to our international conference call.",
        speakerId: '550e8400-e29b-41d4-a716-446655440010',
        language: 'en',
        confidence: 0.93,
        words: this.createWordTimings("Welcome to our international conference call."),
        processingPass: 2,
        createdAt: Date.now()
      },
      {
        id: '550e8400-e29b-41d4-a716-446655440004',
        startTime: 4.5,
        endTime: 9.0,
        text: "こんにちは、今日はお忙しい中お時間をいただきありがとうございます。",
        speakerId: '550e8400-e29b-41d4-a716-446655440011',
        language: 'ja',
        confidence: 0.89,
        words: this.createWordTimings("こんにちは、今日はお忙しい中お時間をいただきありがとうございます。"),
        processingPass: 2,
        createdAt: Date.now() + 4500
      }
    ];
  }

  /**
   * Creates technical presentation segments with specialized vocabulary
   * Tests custom vocabulary handling in future ASR implementations
   */
  static createTechnicalPresentationSegments(): TranscriptionSegment[] {
    const technicalTerms = [
      "machine learning algorithms",
      "neural network architecture", 
      "Docker containerization",
      "Kubernetes orchestration",
      "microservices deployment",
      "API gateway configuration"
    ];
    
    return technicalTerms.map((term, index) => ({
      id: `550e8400-e29b-41d4-a716-44665544${index.toString().padStart(4, '0')}`,
      startTime: index * 8.0,
      endTime: (index + 1) * 8.0 - 0.5,
      text: `Let me explain our approach to ${term} in this project.`,
      speakerId: '550e8400-e29b-41d4-a716-446655440010',
      language: 'en',
      confidence: 0.91,
      words: this.createWordTimings(`Let me explain our approach to ${term} in this project.`),
      processingPass: 2,
      createdAt: Date.now() + (index * 8000)
    }));
  }

  /**
   * Creates realistic speaker profiles for diarization testing
   * Future speaker diarization will need to match these profiles
   */
  static createSpeakerProfiles(): SpeakerProfile[] {
    return [
      {
        id: '550e8400-e29b-41d4-a716-446655440010',
        name: 'Project Manager',
        embedding: this.generateSpeakerEmbedding('male-us-business'),
        languagePreference: 'en',
        totalSpeechTime: 450.5, // seconds
        segmentCount: 25,
        createdAt: Date.now() - 86400000, // 1 day ago
        lastActive: Date.now(),
        isPersistent: true
      },
      {
        id: '550e8400-e29b-41d4-a716-446655440011',
        name: 'Lead Developer',
        embedding: this.generateSpeakerEmbedding('female-us-technical'),
        languagePreference: 'en',
        totalSpeechTime: 380.2,
        segmentCount: 32,
        createdAt: Date.now() - 86400000,
        lastActive: Date.now(),
        isPersistent: true
      },
      {
        id: '550e8400-e29b-41d4-a716-446655440012',
        name: 'Tokyo Team Lead',
        embedding: this.generateSpeakerEmbedding('male-ja-business'),
        languagePreference: 'ja',
        totalSpeechTime: 295.8,
        segmentCount: 18,
        createdAt: Date.now() - 86400000,
        lastActive: Date.now(),
        isPersistent: true
      }
    ];
  }

  /**
   * Creates quality metrics for performance validation
   * Future system monitoring will generate similar metrics
   */
  static createQualityMetrics(): TranscriptionQualityMetrics {
    return {
      overallConfidence: 0.91,
      speakerConsistency: 0.94,
      languageDetectionAccuracy: 0.96,
      realTimeFactor: 0.76,
      memoryUsagePeak: 3.2 * 1024 * 1024 * 1024, // 3.2GB
      cpuUsageAverage: 68.5, // 68.5%
      thermalEvents: [
        {
          timestamp: Date.now() - 1200000,
          temperature: 82.0,
          action: 'throttle',
          durationMs: 45000
        }
      ],
      processingLatency: {
        pass1Average: 1250, // milliseconds
        pass2Average: 3800,
        p95: 2100,
        p99: 4200
      }
    };
  }

  /**
   * Creates two-pass processing test data
   * Validates that Pass 2 improves upon Pass 1 results
   */
  static createTwoPassComparison(): { pass1: TranscriptionSegment[], pass2: TranscriptionSegment[] } {
    const baseText = "The quick brown fox jumps over the lazy dog in the meeting room";
    
    const pass1Segment: TranscriptionSegment = {
      id: '550e8400-e29b-41d4-a716-446655440020',
      startTime: 0.0,
      endTime: 5.0,
      text: "The quik brown fox jumps over the lazy dog in the meting room", // Errors
      speakerId: '550e8400-e29b-41d4-a716-446655440010',
      language: 'en',
      confidence: 0.82, // Lower confidence
      words: this.createWordTimings("The quik brown fox jumps over the lazy dog in the meting room"),
      processingPass: 1,
      createdAt: Date.now()
    };

    const pass2Segment: TranscriptionSegment = {
      ...pass1Segment,
      text: baseText, // Corrected
      confidence: 0.95, // Higher confidence
      words: this.createWordTimings(baseText),
      processingPass: 2,
      updatedAt: Date.now() + 3000
    };

    return {
      pass1: [pass1Segment],
      pass2: [pass2Segment]
    };
  }

  /**
   * Creates error scenarios for edge case testing
   * Tests future error handling capabilities
   */
  static createErrorScenarios(): any[] {
    return [
      {
        scenario: 'audio_too_short',
        input: { duration: 0.1, text: 'Hi' },
        expectedError: 'Audio segment too short for processing'
      },
      {
        scenario: 'no_speech_detected',
        input: { duration: 10, hasVadActivity: false },
        expectedError: 'No speech activity detected'
      },
      {
        scenario: 'unsupported_language',
        input: { language: 'xyz', confidence: 0.9 },
        expectedError: 'Unsupported language code: xyz'
      },
      {
        scenario: 'memory_pressure',
        input: { availableMemory: '1GB', requiredMemory: '4GB' },
        expectedError: 'Insufficient memory for processing'
      },
      {
        scenario: 'thermal_throttle',
        input: { temperature: 95, threshold: 85 },
        expectedError: 'System temperature too high, processing throttled'
      }
    ];
  }

  // Private helper methods

  private static createWordTimings(text: string): WordTiming[] {
    const words = text.split(/\s+/);
    const wordsPerSecond = 2.5; // Average speaking rate
    let currentTime = 0;
    
    return words.map(word => {
      const duration = (word.length / 5) * (1 / wordsPerSecond); // Rough duration
      const wordTiming: WordTiming = {
        word: word.replace(/[.,!?]$/, ''), // Remove trailing punctuation
        startTime: currentTime,
        endTime: currentTime + duration,
        confidence: 0.85 + Math.random() * 0.13 // 0.85-0.98 confidence
      };
      
      currentTime += duration + 0.1; // Small pause between words
      return wordTiming;
    });
  }

  private static generateSpeakerEmbedding(profile: string): number[] {
    // Generate realistic 512-dimensional ECAPA-TDNN speaker embedding
    const embedding = new Array(512);
    const seed = this.hashString(profile);
    
    for (let i = 0; i < 512; i++) {
      // Use deterministic random based on profile for consistent embeddings
      const pseudoRandom = Math.sin(seed + i * 1.618) * 10000;
      embedding[i] = (pseudoRandom - Math.floor(pseudoRandom)) * 2 - 1; // [-1, 1]
    }
    
    // Normalize to unit vector (as ECAPA-TDNN embeddings are)
    const magnitude = Math.sqrt(embedding.reduce((sum, val) => sum + val * val, 0));
    return embedding.map(val => val / magnitude);
  }

  private static hashString(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash);
  }
}

// Test scenario configurations
export const TranscriptionTestScenarios = {
  basicAccuracy: {
    description: 'Basic transcription accuracy test',
    segments: () => TranscriptionTestFactory.createBasicTranscriptionSegments(),
    expectedWER: 0.08, // <8% WER
    expectedConfidence: 0.90
  },
  
  multilingualHandling: {
    description: 'Multilingual meeting handling',
    segments: () => TranscriptionTestFactory.createMultilingualSegments(),
    expectedLanguages: ['en', 'ja'],
    languageDetectionAccuracy: 0.95
  },
  
  technicalVocabulary: {
    description: 'Technical term recognition',
    segments: () => TranscriptionTestFactory.createTechnicalPresentationSegments(),
    customTerms: ['Kubernetes', 'Docker', 'microservices'],
    expectedTermAccuracy: 0.95
  },
  
  speakerDiarization: {
    description: 'Multi-speaker identification',
    profiles: () => TranscriptionTestFactory.createSpeakerProfiles(),
    expectedSpeakerAccuracy: 0.90,
    maxSpeakers: 8
  },
  
  realTimeProcessing: {
    description: 'Real-time transcription performance',
    latencyTarget: 1.5, // seconds
    rtfTarget: 0.8, // Real-time factor
    memoryLimit: 8 * 1024 * 1024 * 1024 // 8GB
  }
};